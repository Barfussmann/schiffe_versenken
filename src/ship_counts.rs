use crate::bit_board::BitBoard;
use crate::board::Board;
use crate::board::Cell;

use super::BOARD_SIZE;
use super::SIZE;
use std::fmt::Display;
use std::fmt::Write;
use std::iter::zip;

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
        for (count, cell) in zip(&mut self.counts, &board.cells) {
            match cell {
                Cell::Ship => {
                    *count += 1;
                }
                Cell::Protected | Cell::Water | Cell::ShipHit => {}
            }
        }
        self.board_count += 1;
    }
    #[inline(never)]
    pub fn add_bit_board(&mut self, board: BitBoard) {
        for i in 0..40 {
            self.counts[i] += ((board.ship()[0] & (1 << i)) != 0) as u64;
        }
        // high bits
        for i in 0..60 {
            self.counts[i + 40] += ((board.ship()[1] & (1 << i)) != 0) as u64;
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

impl Display for ShipCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        for row in self.counts.chunks(SIZE).take(SIZE) {
            for count in row {
                let probability = (*count as f64) / (self.board_count as f64);

                f.write_fmt(format_args!("{:4.1} ", probability * 100.))?;
                // f.write_fmt(format_args!("{:3.1} ", probability * 100.))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
