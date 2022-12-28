use clap::Parser;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{self, Color},
    terminal, Result,
};
use nanorand::Rng;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct ProgArgs {
    /// Number of bots to create
    #[arg(short, long, default_value_t = 10, value_name = "BOTS")]
    bots: u8,

    /// Tick delay in msec (inverse of speed)
    #[arg(short, long, default_value_t = 250u128, value_name = "MSEC")]
    tick_delay: u128,

    /// Sight (how many squares ahead a bot can "see")
    #[arg(short, long, default_value_t = 10, value_name = "SQUARES")]
    sight: u16,

    /// Vegetation regrowth time (in ticks)
    #[arg(short, long, default_value_t = 100, value_name = "TICKS")]
    regrow_time: u16,

    /// Map vegetation probability (in percent)
    #[arg(short, long, default_value_t = 5, value_name = "PERCENT")]
    food_prob: u8,
}

#[derive(Clone, Copy)]
enum NekobotState {
    Wander,
    Forage,
    Dead,
}

struct Nekobot {
    label: String,
    row: u16,
    col: u16,
    energy: u8,
    sight: u16,
    state: NekobotState,
}

enum NekoDirs {
    Up,
    Down,
    Left,
    Right,
}

struct NystopiaTile {
    has_food: bool,
    eaten: bool,
    regrowth_counter: u16,
    regrowth_rate: u16,
}

impl NystopiaTile {
    pub fn new(prog_args: &ProgArgs) -> Self {
        let mut rng = nanorand::tls_rng();
        if (rng.generate::<u8>() % 100) < prog_args.food_prob {
            // It's a food tile
            Self {
                has_food: true,
                eaten: false,
                regrowth_counter: 0u16,
                regrowth_rate: prog_args.regrow_time,
            }
        } else {
            // It's not a food tile
            Self {
                has_food: false,
                eaten: false,
                regrowth_counter: 0u16,
                regrowth_rate: 0,
            }
        }
    }
}

struct NystopiaMap {
    map: Vec<NystopiaTile>,
    cols: u16,
    rows: u16,
}

impl NystopiaMap {
    pub fn new(prog_args: &ProgArgs) -> Result<Self> {
        let (my_cols, my_rows) = terminal::size()?;
        let mut new_map = vec![];

        for _ in 0..(my_cols * my_rows) {
            new_map.push(NystopiaTile::new(prog_args));
        }

        Ok(Self {
            cols: my_cols,
            rows: my_rows,
            map: new_map,
        })
    }

    pub fn get_cols(self: &Self) -> u16 {
        self.cols
    }

    pub fn get_rows(self: &Self) -> u16 {
        self.rows
    }

    pub fn get_tile(self: &Self, row: u16, col: u16) -> Option<&NystopiaTile> {
        if row < self.get_rows() && col < self.get_cols() {
            Some(&self.map[(self.get_cols() * row + col) as usize])
        } else {
            None
        }
    }

    pub fn consume(self: &mut Self, row: u16, col: u16) -> bool {
        if row < self.get_rows() && col < self.get_cols() {
            let index = self.get_cols() * row + col;
            let tile = &mut self.map[index as usize];
            if tile.has_food && !tile.eaten {
                tile.eaten = true;
                tile.regrowth_counter = tile.regrowth_rate;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn tick_map(self: &mut Self) {
        for r in 0..self.get_rows() {
            for c in 0..self.get_cols() {
                let index = self.get_cols() * r + c;
                let tile = &mut self.map[index as usize];
                if tile.has_food && tile.eaten {
                    if tile.regrowth_counter == 1 {
                        tile.eaten = false;
                    }
                    tile.regrowth_counter -= 1;
                }
            }
        }
    }

    pub fn render_map(self: &Self, stdout: &mut std::io::Stdout) -> Result<()> {
        queue!(stdout, cursor::MoveTo(0, 0))?;

        for r in 0..self.get_rows() {
            for c in 0..self.get_cols() {
                if let Some(this_tile) = self.get_tile(r, c) {
                    if this_tile.has_food && !this_tile.eaten {
                        queue!(
                            stdout,
                            style::SetBackgroundColor(Color::DarkGreen),
                            style::Print(" ")
                        )?;
                    } else {
                        queue!(
                            stdout,
                            style::SetBackgroundColor(Color::Black),
                            style::Print(" ")
                        )?;
                    }
                }
            }
        }

        stdout.flush()?;

        Ok(())
    }
}

impl Nekobot {
    fn new_rand(label: &str, rows: u16, cols: u16, prog_args: &ProgArgs) -> Self {
        let mut rng = nanorand::tls_rng();
        Self {
            row: rng.generate::<u16>() % rows,
            col: rng.generate::<u16>() % cols,
            label: label.into(),
            energy: 100,
            sight: prog_args.sight,
            state: NekobotState::Wander,
        }
    }

    fn get_state(self: &Self) -> NekobotState {
        self.state
    }

    fn move_it(self: &mut Self, dir: &NekoDirs, map: &NystopiaMap) {
        match dir {
            NekoDirs::Up => self.row = if self.row == 0 { 0 } else { self.row - 1 },
            NekoDirs::Down => self.row = (map.get_rows() - 1).min(self.row + 1),
            NekoDirs::Left => self.col = if self.col == 0 { 0 } else { self.col - 1 },
            NekoDirs::Right => self.col = (map.get_cols() - 1).min(self.col + 1),
        }
    }

    fn tick(self: &mut Self, map: &mut NystopiaMap) {
        if self.energy > 0 {
            let mut rng = nanorand::tls_rng();
            let dirs = [
                NekoDirs::Up,
                NekoDirs::Down,
                NekoDirs::Left,
                NekoDirs::Right,
            ];
            self.state = NekobotState::Wander;
            if self.get_energy() >= 80 || (!self.eat(map) && !self.forage(map)) {
                self.move_it(&dirs[(rng.generate::<u8>() % 4) as usize], map);
            }
            self.energy -= 1;
        } else {
            self.state = NekobotState::Dead;
        }
    }

    fn get_energy(self: &Self) -> u8 {
        self.energy
    }

    fn eat(self: &mut Self, map: &mut NystopiaMap) -> bool {
        if map.consume(self.row, self.col) {
            self.energy = 100;
            true
        } else {
            false
        }
    }

    fn sight_dims(self: &Self, map: &NystopiaMap) -> (u16, u16, u16, u16) {
        let left = if self.col > self.sight {
            self.col - self.sight
        } else {
            0
        };
        let right = map.get_cols().min(self.col + self.sight);
        let top = if self.row > self.sight {
            self.row - self.sight
        } else {
            0
        };
        let bottom = map.get_rows().min(self.row + self.sight);

        (left, right, top, bottom)
    }

    fn forage(self: &mut Self, map: &NystopiaMap) -> bool {
        let (left, right, top, bottom) = self.sight_dims(map);

        let mut nearest_row = 0;
        let mut nearest_col = 0;
        let mut nearest_dist = self.sight * 2;

        self.state = NekobotState::Forage;

        for row in top..bottom {
            for col in left..right {
                if let Some(tile) = map.get_tile(row, col) {
                    let aside = row.max(self.row) - row.min(self.row);
                    let bside = col.max(self.col) - col.min(self.col);
                    let cside = (((aside * aside) + (bside * bside)) as f64).sqrt() as u16;

                    if cside <= self.sight && tile.has_food && !tile.eaten {
                        if (cside < nearest_dist)
                            || ((cside == nearest_dist)
                                && (nanorand::tls_rng().generate::<u8>() % 2) == 1)
                        {
                            nearest_dist = cside;
                            nearest_row = row;
                            nearest_col = col;
                        }
                    }
                }
            }
        }

        if nearest_dist <= self.sight {
            if nearest_row < self.row {
                if nearest_col < self.col {
                    if self.col - nearest_col < self.row - nearest_row {
                        self.move_it(&NekoDirs::Up, map);
                    } else {
                        self.move_it(&NekoDirs::Left, map);
                    }
                } else {
                    if nearest_col - self.col < self.row - nearest_row {
                        self.move_it(&NekoDirs::Up, map);
                    } else {
                        self.move_it(&NekoDirs::Right, map);
                    }
                }
            } else {
                if nearest_col < self.col {
                    if self.col - nearest_col < nearest_row - self.row {
                        self.move_it(&NekoDirs::Down, map);
                    } else {
                        self.move_it(&NekoDirs::Left, map);
                    }
                } else {
                    if nearest_col - self.col < nearest_row - self.row {
                        self.move_it(&NekoDirs::Down, map);
                    } else {
                        self.move_it(&NekoDirs::Right, map);
                    }
                }
            }
            true
        } else {
            false
        }
    }
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    let prog_args = ProgArgs::parse();
    let mut nekomap = NystopiaMap::new(&prog_args)?;

    let mut count = prog_args.bots as usize;
    let period = prog_args.tick_delay;

    let labels = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()";

    let mut nekobots: Vec<Nekobot> = vec![];

    while count > 0 {
        let label_index = count % labels.len();
        nekobots.push(Nekobot::new_rand(
            format!("{}", labels.get(label_index..(label_index + 1)).unwrap()).as_str(),
            nekomap.get_rows(),
            nekomap.get_cols(),
            &prog_args,
        ));
        count -= 1;
    }

    terminal::enable_raw_mode()?;

    // Clear terminal
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )?;

    let inst = Instant::now();
    let mut last_capture = inst.elapsed().as_millis();
    let mut next_stop = last_capture + period;

    loop {
        last_capture = inst.elapsed().as_millis();
        if last_capture >= next_stop {
            // Render the map
            nekomap.tick_map();
            nekomap.render_map(&mut stdout)?;

            // Render the bots
            for nekobot in nekobots.iter_mut() {
                let mut tile = nekomap
                    .get_tile(nekobot.row, nekobot.col)
                    .expect("Failed to identify tile".into());
                // Remove the old placements
                queue!(
                    stdout,
                    cursor::MoveTo(nekobot.col, nekobot.row),
                    if !tile.eaten && tile.has_food {
                        style::SetBackgroundColor(Color::DarkGreen)
                    } else {
                        style::SetBackgroundColor(Color::Black)
                    },
                    style::Print(" ")
                )?;

                nekobot.tick(&mut nekomap);

                tile = nekomap
                    .get_tile(nekobot.row, nekobot.col)
                    .expect("Failed to identify tile".into());

                // Draw the new placements
                queue!(
                    stdout,
                    cursor::MoveTo(nekobot.col, nekobot.row),
                    if !tile.eaten && tile.has_food {
                        style::SetBackgroundColor(Color::DarkGreen)
                    } else {
                        style::SetBackgroundColor(Color::Black)
                    },
                    match nekobot.get_state() {
                        NekobotState::Wander => style::SetForegroundColor(Color::Grey),
                        NekobotState::Forage => style::SetForegroundColor(Color::Yellow),
                        NekobotState::Dead => style::SetForegroundColor(Color::Red),
                    },
                    style::Print(nekobot.label.clone())
                )?;
            }

            next_stop = last_capture + period;
        }

        // Flush the output buffer
        stdout.flush()?;

        match poll(Duration::from_millis((next_stop - last_capture) as u64)) {
            Ok(true) => match read() {
                Ok(Event::Key(ev)) => {
                    if ev.code == KeyCode::Char('q') {
                        break;
                    }
                }
                _ => {}
            },
            Ok(false) => {}
            Err(_) => {}
        }
    }

    terminal::disable_raw_mode()?;

    // Put cursor at bottom-left before exit
    queue!(stdout, cursor::MoveTo(0, nekomap.get_rows() - 1))?;
    queue!(stdout, cursor::Show)?;
    stdout.flush()?;

    Ok(())
}
