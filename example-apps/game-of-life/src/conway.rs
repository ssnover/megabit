use rand::prelude::*;

#[derive(Clone, Debug)]
pub struct BoardState {
    pub state: Vec<u8>,
    pub height: usize,
    pub width: usize,
}

impl BoardState {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            state: vec![0u8; (width / 8) * height],
            height,
            width,
        }
    }

    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        for elem in self.state.iter_mut() {
            for bit in 0..8 {
                let val = rng.gen_range(0..4);
                if val == 3 {
                    *elem |= 1 << bit;
                }
            }
        }
    }

    fn get_cell(&self, col: usize, row: usize) -> bool {
        (self.state[((self.width / 8) * row) + (col / 8)] & (1 << (col % 8))) != 0
    }

    fn set_cell(&mut self, col: usize, row: usize, val: bool) {
        if val {
            self.state[((self.width / 8) * row) + (col / 8)] |= 1 << (col % 8);
        } else {
            self.state[((self.width / 8) * row) + (col / 8)] &= !(1 << (col % 8));
        }
    }

    pub fn step(&mut self, prev: &Self) {
        for col in 0..self.width {
            for row in 0..self.height {
                match (prev.get_cell(col, row), prev.neighbors_alive(col, row)) {
                    (true, 2..=3) => self.set_cell(col, row, true),
                    (false, 3) => self.set_cell(col, row, true),
                    (true, _) => self.set_cell(col, row, false),
                    (false, _) => self.set_cell(col, row, false),
                }
            }
        }
    }

    fn neighbors_alive(&self, col: usize, row: usize) -> u8 {
        let lower_x = col.saturating_sub(1);
        let upper_x = core::cmp::min(col + 1, self.width - 1);
        let lower_y = row.saturating_sub(1);
        let upper_y = core::cmp::min(row + 1, self.height - 1);

        let mut count = 0;
        for x in lower_x..=upper_x {
            for y in lower_y..=upper_y {
                if col == x && row == y {
                    continue;
                }
                if self.get_cell(x, y) {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn count_live_population(&self) -> u32 {
        self.state
            .iter()
            .map(|elem| {
                (0..8)
                    .into_iter()
                    .map(|bit| if (*elem & (1 << bit)) != 0 { 1 } else { 0 })
                    .sum::<u32>()
            })
            .sum()
    }
}
