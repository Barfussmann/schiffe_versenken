use core::simd::{simd_swizzle, u64x4};
use std::simd::prelude::*;
#[allow(unused)]
use std::{
    arch::x86_64::{_mm256_popcnt_epi64, _mm512_popcnt_epi64},
    mem::transmute,
    simd::{Mask, Swizzle, cmp::SimdPartialOrd, num::SimdUint},
};

use crate::{bit_iter::BitIter, board::Board, solver::PlacedBitShips};
use crate::{board::Cell, ship::Ship};
#[derive(Clone, Copy)]
#[repr(align(256))]
pub struct BitBoard {
    pub protected_and_ship: [u64x8; 3],
}
impl BitBoard {
    pub const INSTRUCTION_PARALLELISM: usize = 1;
    pub fn allowable<const S: Ship>(&self) -> u64x4 {
        match S.length() {
            1 => simd_swizzle!(self.protected_and_ship[0], [2, 3, 2, 3]),
            2 => simd_swizzle!(self.protected_and_ship[0], [4, 5, 6, 7]),
            3 => simd_swizzle!(self.protected_and_ship[1], [0, 1, 2, 3]),
            4 => simd_swizzle!(self.protected_and_ship[1], [4, 5, 6, 7]),
            5 => simd_swizzle!(self.protected_and_ship[2], [0, 1, 2, 3]),
            6 => simd_swizzle!(self.protected_and_ship[2], [4, 5, 6, 7]),
            _ => unreachable!("Invalid ship length"),
        }
    }
    pub fn allowable_dyn(&self, ship: Ship) -> u64x4 {
        match ship.length() {
            1 => simd_swizzle!(self.protected_and_ship[0], [2, 3, 2, 3]),
            2 => simd_swizzle!(self.protected_and_ship[0], [4, 5, 6, 7]),
            3 => simd_swizzle!(self.protected_and_ship[1], [0, 1, 2, 3]),
            4 => simd_swizzle!(self.protected_and_ship[1], [4, 5, 6, 7]),
            5 => simd_swizzle!(self.protected_and_ship[2], [0, 1, 2, 3]),
            6 => simd_swizzle!(self.protected_and_ship[2], [4, 5, 6, 7]),
            _ => unreachable!("Invalid ship length"),
        }
    }
    pub fn ship(&self) -> u64x2 {
        !simd_swizzle!(self.protected_and_ship[0], [0, 1])
    }

    pub fn new(board: Board) -> Self {
        let ship = !board.to_u64x2(Cell::Ship);

        let protected = board.to_protected();

        // let protected_1 = protected.to_u64x2(Cell::Protected);

        let protected_1 = protected.shifted_protected::<{ Ship::new(1, 0) }>(); // x and y are the same so we only need one
        let protected_2 = protected.shifted_protected::<{ Ship::new(2, 0) }>();
        let protected_3 = protected.shifted_protected::<{ Ship::new(3, 0) }>();
        let protected_4 = protected.shifted_protected::<{ Ship::new(4, 0) }>();
        let protected_5 = protected.shifted_protected::<{ Ship::new(5, 0) }>();
        let protected_6 = protected.shifted_protected::<{ Ship::new(6, 0) }>();

        Self {
            protected_and_ship: [
                simd_swizzle!(
                    u64x4::from_array([ship[0], ship[1], protected_1[0], protected_1[1]]),
                    protected_2,
                    [0, 1, 2, 3, 4, 5, 6, 7]
                ),
                simd_swizzle!(protected_3, protected_4, [0, 1, 2, 3, 4, 5, 6, 7]),
                simd_swizzle!(protected_5, protected_6, [0, 1, 2, 3, 4, 5, 6, 7]),
            ],
        }
    }
    // #[inline(never)]
    pub fn place_ship_dyn(&mut self, ship: Ship, index: usize, placed_bit_ships: &PlacedBitShips) {
        let placed_ship_board = unsafe {
            placed_bit_ships
                .placed_ships
                .get_unchecked(ship.index())
                .get_unchecked(index)
        };
        // we only need the ships that are shorter than the current ship
        for i in 0..ship.length().div_ceil(2) {
            unsafe {
                *self.protected_and_ship.get_unchecked_mut(i) &=
                    *placed_ship_board.protected_and_ship.get_unchecked(i);
            }
            // self.protected_and_ship[i] &= placed_ship_board.protected_and_ship[i];
        }
    }
    pub fn place_ship<const S: Ship>(&mut self, index: u8, placed_bit_ships: &PlacedBitShips) {
        let placed_ship_board = unsafe {
            placed_bit_ships
                .placed_ships
                .get_unchecked(S.index())
                .get_unchecked(index as usize)
        };
        // we only need the ships that are shorter than the current ship
        for i in 0..S.length().div_ceil(2) {
            self.protected_and_ship[i] &= placed_ship_board.protected_and_ship[i];
        }
    }
    pub fn placed_ships_to_board(&self) -> Board {
        let mut board = Board::new();

        for set_ship in BitIter::new(simd_swizzle!(self.ship(), [0, 1, 0, 1])) {
            if set_ship >= 128 {
                break;
            }
            board.cells[set_ship as usize] = Cell::Ship;
        }
        board
    }
}
