use super::ProgArgs;
use nanorand::Rng;

pub struct NystopiaTile {
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

    pub fn has_food(self: &Self) -> bool {
        self.has_food
    }

    pub fn eaten(self: &Self) -> bool {
        self.eaten
    }
}

pub struct NystopiaMap {
    map: Vec<NystopiaTile>,
    cols: u16,
    rows: u16,
}

impl NystopiaMap {
    pub fn new(
        prog_args: &ProgArgs,
        my_cols: u16,
        my_rows: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
}
