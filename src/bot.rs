use super::map::NystopiaMap;
use super::ProgArgs;
use nanorand::Rng;
use log::info;

#[derive(Clone, Copy)]
pub enum NekobotState {
    Wander,
    Forage,
    Dead,
}

pub struct Nekobot {
    label: String,
    row: u16,
    col: u16,
    energy: u8,
    sight: u16,
    see_food_move_score: u64,
    move_score: u64,
    state: NekobotState,
    hungry_threshold: u8,
    nekode: Vec<NekoOps>,
}

#[derive(Clone)]
enum NekoDirs {
    Here,
    Up,
    Down,
    Left,
    Right,
}

impl std::fmt::Display for NekoDirs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Here => write!(f, "Here"),
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
        }
    }
}

#[derive(Clone)]
enum NekoOps {
    Sense(NekoDirs),
    Move,
    Eat,
    Halt,
}

impl Nekobot {
    pub fn new_rand(label: &str, rows: u16, cols: u16, prog_args: &ProgArgs) -> Self {
        let mut rng = nanorand::tls_rng();
        Self {
            row: rng.generate::<u16>() % rows,
            col: rng.generate::<u16>() % cols,
            label: label.into(),
            energy: rng.generate::<u8>() % 90 + 10,
            sight: prog_args.sight,
            state: NekobotState::Wander,
            see_food_move_score: 200000,
            move_score: 100,
            hungry_threshold: 80,
            nekode: vec![
                NekoOps::Sense(NekoDirs::Here),
                NekoOps::Sense(NekoDirs::Up),
                NekoOps::Sense(NekoDirs::Right),
                NekoOps::Sense(NekoDirs::Down),
                NekoOps::Sense(NekoDirs::Left),
                NekoOps::Move,
                NekoOps::Eat,
                NekoOps::Halt,
            ]
        }
    }

    pub fn get_state(self: &Self) -> NekobotState {
        self.state
    }

    fn op_sense(&self, membank: &mut Vec<u64>, map: &NystopiaMap, dir: &NekoDirs) {
        if self.hungry() {
            membank.push(self.forage2(map, dir));
        } else {
            membank.push(self.move_score);
        }
    }

    fn op_move(&mut self, membank: &mut Vec<u64>, map: &NystopiaMap) {
        let mut rng = nanorand::tls_rng();

        // Since Vec works like a stack, these need to be in reverse-order compared to the
        // list in self.nekode
        let dirs = [
            NekoDirs::Left,
            NekoDirs::Down,
            NekoDirs::Right,
            NekoDirs::Up,
            NekoDirs::Here,
        ];
        let max_score: u64 = membank.iter().sum();
        let mut rnd_score: u64 = rng.generate::<u64>() % max_score;
        info!("Max({}) Chose({}) Membank({})", max_score, rnd_score, membank.iter().map(|x| format!("{}", x)).collect::<Vec<String>>().join(","));
        for v in dirs {
            if let Some(next_score) = membank.pop() {
                if rnd_score >= next_score {
                    rnd_score -= next_score;
                } else {
                    info!("Moving: {}", v);
                    self.move_it(&v, map);
                    break;
                }
            }
        }
        membank.clear();
    }

    fn op_eat(&mut self, map: &mut NystopiaMap) {
        if !self.hungry() {
            return;
        }
        self.eat(map);
    }

    fn move_it(self: &mut Self, dir: &NekoDirs, map: &NystopiaMap) {
        match dir {
            NekoDirs::Here => {},
            NekoDirs::Up => self.row = if self.row == 0 { 0 } else { self.row - 1 },
            NekoDirs::Down => self.row = (map.get_rows() - 1).min(self.row + 1),
            NekoDirs::Left => self.col = if self.col == 0 { 0 } else { self.col - 1 },
            NekoDirs::Right => self.col = (map.get_cols() - 1).min(self.col + 1),
        }
    }

    fn hungry(&self) -> bool {
        self.get_energy() < self.hungry_threshold
    }

    pub fn tick(self: &mut Self, map: &mut NystopiaMap) {
        if self.energy > 0 {
            let mut membank: Vec<u64> = vec![];
            for op in self.nekode.clone().iter() {
                match op {
                    NekoOps::Sense(dir) => self.op_sense(&mut membank, map, dir),
                    NekoOps::Move => self.op_move(&mut membank, map),
                    NekoOps::Eat => self.op_eat(map),
                    NekoOps::Halt => break,
                }
            }
            self.energy -= 1;
            if self.hungry() {
                self.state = NekobotState::Forage;
            } else {
                self.state = NekobotState::Wander;
            }
        } else {
            self.state = NekobotState::Dead;
        }
    }

    pub fn tick_old(self: &mut Self, map: &mut NystopiaMap) {
        if self.energy > 0 {
            let mut rng = nanorand::tls_rng();
            let dirs = [
                NekoDirs::Up,
                NekoDirs::Down,
                NekoDirs::Left,
                NekoDirs::Right,
            ];
            self.state = NekobotState::Wander;
            if !self.hungry() || (!self.eat(map) && !self.forage(map)) {
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
            self.energy += 20;
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

    fn compute_food_move_score(&self, food_row: u64, food_col: u64) -> u64 {
        let side_a = (self.row as f64) - (food_row as f64);
        let side_b = (self.col as f64) - (food_col as f64);
        let side_c = side_a.abs().hypot(side_b.abs()).min(self.sight as f64);
        let score = (self.see_food_move_score as f64)*(self.sight as f64 - side_c)/(self.sight as f64);
        score.round() as u64 + self.move_score
    }

    fn forage2(self: &Self, map: &NystopiaMap, dir: &NekoDirs) -> u64 {
        // If there's food here, then grant it the max score
        if let NekoDirs::Here = dir {
            if let Some(tile) = map.get_tile(self.row, self.col) {
                if tile.has_food() {
                    return self.see_food_move_score;
                } else {
                    return self.move_score;
                }
            }
        }

        let (left, right, top, bottom) = self.sight_dims(map);

        let mut nearest_row = 0;
        let mut nearest_col = 0;
        let mut nearest_dist = self.sight * 2;

        for row in top..bottom {
            for col in left..right {
                if let Some(tile) = map.get_tile(row, col) {
                    let aside = row.max(self.row) - row.min(self.row);
                    let bside = col.max(self.col) - col.min(self.col);
                    let cside = (((aside * aside) + (bside * bside)) as f64).sqrt() as u16;

                    if cside <= self.sight && tile.has_food() && !tile.eaten() {
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
                        if let NekoDirs::Up = dir {
                            return self.compute_food_move_score(nearest_row as u64, nearest_col as u64);
                        }
                    } else {
                        if let NekoDirs::Left = dir {
                            return self.compute_food_move_score(nearest_row as u64, nearest_col as u64);
                        }
                    }
                } else {
                    if nearest_col - self.col < self.row - nearest_row {
                        if let NekoDirs::Up = dir {
                            return self.compute_food_move_score(nearest_row as u64, nearest_col as u64);
                        }
                    } else {
                        if let NekoDirs::Right = dir {
                            return self.compute_food_move_score(nearest_row as u64, nearest_col as u64);
                        }
                    }
                }
            } else {
                if nearest_col < self.col {
                    if self.col - nearest_col < nearest_row - self.row {
                        if let NekoDirs::Down = dir {
                            return self.compute_food_move_score(nearest_row as u64, nearest_col as u64);
                        }
                    } else {
                        if let NekoDirs::Left = dir {
                            return self.compute_food_move_score(nearest_row as u64, nearest_col as u64);
                        }
                    }
                } else {
                    if nearest_col - self.col < nearest_row - self.row {
                        if let NekoDirs::Down = dir {
                            return self.compute_food_move_score(nearest_row as u64, nearest_col as u64);
                        }
                    } else {
                        if let NekoDirs::Right = dir {
                            return self.compute_food_move_score(nearest_row as u64, nearest_col as u64);
                        }
                    }
                }
            }
        }
        self.move_score
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

                    if cside <= self.sight && tile.has_food() && !tile.eaten() {
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

    pub fn get_row(self: &Self) -> u16 {
        self.row
    }

    pub fn get_col(self: &Self) -> u16 {
        self.col
    }

    pub fn get_label(self: &Self) -> &String {
        &(self.label)
    }
}
