use crate::bit_board::BitBoard;
use crate::bit_board::DoubleBitBoard;
use crate::board::Board;
use crate::board::Cell;

use super::BOARD_SIZE;
use super::SIZE;
use std::fmt::Display;
use std::fmt::Write;
use std::iter::zip;
use std::simd::mask8x64;
use std::simd::num::SimdInt;
use std::simd::u8x64;

#[derive(Debug, Clone)]
pub struct ShipCountsSmall {
    small_counts: [u8x64; 2],
}
impl ShipCountsSmall {
    pub fn new() -> ShipCountsSmall {
        ShipCountsSmall {
            small_counts: [u8x64::splat(0); 2],
        }
    }
    pub fn add_double_bit_board(&mut self, board: DoubleBitBoard) {
        self.add_bit_board(board.boards[0]);
        self.add_bit_board(board.boards[1]);
    }
    // #[inline(never)]
    pub fn add_bit_board(&mut self, board: BitBoard) {
        let ship = !board.ship();

        self.small_counts[0] -= mask8x64::from_bitmask(ship[0]).to_int().cast();
        self.small_counts[1] -= mask8x64::from_bitmask(ship[1]).to_int().cast();
    }
}

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
    pub fn add_small_counts(&mut self, small_counts: ShipCountsSmall, count: u64) {
        for i in 0..40 {
            self.counts[i] += small_counts.small_counts[0][i] as u64;
        }
        // high bits
        for i in 0..60 {
            self.counts[i + 40] += small_counts.small_counts[1][i] as u64;
        }
        assert!(
            count < 256,
            "added count is to big. The result could be wrong"
        );
        self.board_count += count;
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
