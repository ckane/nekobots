use nanorand::Rng;
use super::map::NystopiaMap;
use super::ProgArgs;

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
    state: NekobotState,
}

enum NekoDirs {
    Up,
    Down,
    Left,
    Right,
}

impl Nekobot {
    pub fn new_rand(label: &str, rows: u16, cols: u16, prog_args: &ProgArgs) -> Self {
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

    pub fn get_state(self: &Self) -> NekobotState {
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

    pub fn tick(self: &mut Self, map: &mut NystopiaMap) {
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
