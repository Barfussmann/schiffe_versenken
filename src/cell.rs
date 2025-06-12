use std::{
    fmt::Display,
    ops::{Index, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cell {
    Water = 0,
    Protected = 1,
    ShipHit = 2,
    Ship = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct CellCount {
    counts: [u64; 4],
}
impl CellCount {
    pub fn new() -> Self {
        Self { counts: [0; 4] }
    }
    pub fn add_cell(&mut self, cell: Cell) {
        self.counts[cell as usize] += 1;
    }
}

impl Index<Cell> for CellCount {
    type Output = u64;

    fn index(&self, index: Cell) -> &Self::Output {
        &self.counts[index as usize]
    }
}
impl Sub<CellCount> for CellCount {
    type Output = CellCount;

    fn sub(self, rhs: CellCount) -> Self::Output {
        CellCount {
            counts: [
                self.counts[0] - rhs.counts[0],
                self.counts[1] - rhs.counts[1],
                self.counts[2] - rhs.counts[2],
                self.counts[3] - rhs.counts[3],
            ],
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Cell::Protected => " o ",
            Cell::Water => " _ ",
            Cell::ShipHit | Cell::Ship => " X ",
        };
        f.write_str(str)
    }
}
