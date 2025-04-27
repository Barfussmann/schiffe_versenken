use crate::{
    bit_board::{BitBoard, DoubleBitBoard, OctaBitBoard},
    board::{Board, Cell},
    ship::Ship,
};

use super::BOARD_SIZE;
use super::SIZE;
use core::{
    convert::TryInto,
    simd::{num::SimdUint, u64x4, u64x64},
};
use std::{
    fmt::{Display, Write},
    iter::zip,
    simd::prelude::*,
};

#[derive(Debug, Clone)]
pub struct ShipPositionCounts {
    counts: [u64x64; 4],
    added_boards: u64,
}
impl ShipPositionCounts {
    pub fn new() -> ShipPositionCounts {
        ShipPositionCounts {
            counts: [u64x64::splat(0); 4],
            added_boards: 0,
        }
    }
    fn counts(&self) -> [u64; 256] {
        self.counts
            .map(|x| x.to_array())
            .as_flattened()
            .try_into()
            .unwrap()
    }
    pub fn add_single_ship(&mut self, index: u8, counts: u64) {
        let u64_index = index as usize / 64;
        let rem_index = index as usize % 64;
        self.counts[u64_index][rem_index] += counts;
    }
    pub fn add_small(&mut self, small_counts: &mut ShipPositionCountsSmall) {
        for i in 0..4 {
            self.counts[i] += small_counts.small_counts[i].cast();
        }
        *small_counts = ShipPositionCountsSmall::new();
    }
}
#[derive(Debug, Clone)]
pub struct ShipPositionCountsSmall {
    small_counts: [u8x64; 4],
}
impl ShipPositionCountsSmall {
    pub fn new() -> ShipPositionCountsSmall {
        ShipPositionCountsSmall {
            small_counts: [u8x64::splat(0); 4],
        }
    }
    // returns the count of the added ships
    pub fn add_possible_ship_positions(&mut self, ship_positions: u64x4) -> u64 {
        // let added_ships: u64x4 = unsafe { _mm256_popcnt_epi64(ship_positions.into()).into() };

        let mut added_ships = 0;
        for i in 0..4 {
            added_ships += ship_positions[i].count_ones() as u64;
            self.small_counts[i] -= mask8x64::from_bitmask(ship_positions[i]).to_int().cast();
        }
        added_ships
        // added_ships.reduce_sum()
    }
}

#[derive(Debug, Clone)]
pub struct ShipCountsSmall {
    small_counts: [u8x64; 2],
    pub added_ships: u64,
}
impl ShipCountsSmall {
    pub fn new() -> ShipCountsSmall {
        ShipCountsSmall {
            small_counts: [u8x64::splat(0); 2],
            added_ships: 0,
        }
    }
    pub fn add_octa_bit_board(&mut self, board: OctaBitBoard) {
        for bit_board in board.boards {
            self.add_bit_board(bit_board);
        }
    }
    pub fn add_double_bit_board(&mut self, board: DoubleBitBoard) {
        self.add_bit_board(board.boards[0]);
        self.add_bit_board(board.boards[1]);
    }
    // #[inline(never)]
    pub fn add_bit_board(&mut self, board: BitBoard) {
        self.added_ships += 1;
        let ship = board.ship();
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
    // #[inline(never)]
    pub fn add_small_counts(&mut self, small_counts: &mut ShipCountsSmall) {
        for i in 0..64 {
            self.counts[i] += small_counts.small_counts[0][i] as u64;
        }
        // high bits
        for i in 0..36 {
            self.counts[i + 64] += small_counts.small_counts[1][i] as u64;
        }
        self.board_count += small_counts.added_ships;
        assert!(
            small_counts.added_ships < 256,
            "added count is to big. The result could be wrong"
        );
        *small_counts = ShipCountsSmall::new();
    }
    // #[inline(never)]
    pub fn add_ship_positions_counts<const SHIP: Ship>(
        &mut self,
        ship_counts: &mut ShipPositionCounts,
    ) {
        let counts = ship_counts.counts();
        for i in 0..100 {
            let ship_count_x = counts[i];
            let ship_count_y = counts[i + 128];
            for ship_i in 0..SHIP.length() {
                self.counts[i + ship_i] += ship_count_x;
                if i + ship_i * 10 < 128 {
                    self.counts[i + ship_i * 10] += ship_count_y;
                }
            }
        }
        // self.board_count += ship_counts.added_boards;
        *ship_counts = ShipPositionCounts::new();
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
