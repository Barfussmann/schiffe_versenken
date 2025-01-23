use std::{fmt::Write, iter::zip};

use crate::{
    BOARD_SIZE, SIZE,
    board::{Board, Cell},
};

#[derive(Debug, Clone)]
pub struct ShipCounts {
    pub counts: [u64; BOARD_SIZE],
    pub board_count: u64,
}

impl ShipCounts {
    pub fn new() -> ShipCounts {
        ShipCounts {
            counts: [0; BOARD_SIZE],
            board_count: 0,
        }
    }
    pub fn add_board(&mut self, board: Board) {
        for (count, cell) in zip(&mut self.counts, board.cells.as_flattened()) {
            match cell {
                Cell::Ship => {
                    *count += 1;
                }
                Cell::Protected | Cell::Water | Cell::ShipHit => {}
            }
        }
        self.board_count += 1;
    }
    pub fn add_other_count(&mut self, other: Self) {
        for (self_count, other_count) in zip(&mut self.counts, &other.counts) {
            *self_count += *other_count;
        }
        self.board_count += other.board_count;
    }
}
impl std::fmt::Display for ShipCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        // for row in self.counts.chunks(16) {
        //     for count in row {
        for row in self.counts.chunks(16).take(SIZE) {
            for count in &row[..SIZE] {
                let probability = (*count as f64) / (self.board_count as f64);

                f.write_fmt(format_args!("{:4.3} ", probability))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
