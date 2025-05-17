use core::simd::{simd_swizzle, u64x4};
#[allow(unused)]
use std::{
    arch::x86_64::{_mm256_popcnt_epi64, _mm512_popcnt_epi64},
    mem::transmute,
    simd::{Mask, Swizzle, cmp::SimdPartialOrd, num::SimdUint},
};
use std::{simd::prelude::*, sync::LazyLock};

use crate::{
    SIZE,
    bit_iter::BitIter,
    board::{Board, Direction},
};
use crate::{board::Cell, ship::Ship};
#[derive(Clone, Copy)]
#[repr(align(256))]
pub struct BitBoard {
    pub protected_and_ship: [u64x8; 3],
}
impl BitBoard {
    pub const INSTRUCTION_PARALLELISM: usize = 1;
    pub fn allowable<const SHIP_INDEX: usize>(&self) -> u64x4 {
        match Ship::length_from_index(SHIP_INDEX) {
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
    #[must_use]
    pub fn place_ship<const SHIP_INDEX: usize>(
        self,
        index: u8,
        placed_bit_ships: &PlacedBitShips,
    ) -> Self {
        let mut this = self;
        let placed_ship_board = unsafe {
            placed_bit_ships
                .placed_ships
                .get_unchecked(SHIP_INDEX)
                .get_unchecked(index as usize)
        };
        // we only need the ships that are shorter than the current ship
        for i in 0..Ship::length_from_index(SHIP_INDEX).div_ceil(2) {
            this.protected_and_ship[i] &= placed_ship_board.protected_and_ship[i];
        }
        this
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

pub struct PlacedBitShips {
    pub placed_ships: [[BitBoard; 256]; 5],
}
impl PlacedBitShips {
    pub fn new() -> &'static Self {
        const SHIPS: [Ship; 5] = [
            Ship::new(5, 0),
            Ship::new(4, 0),
            Ship::new(3, 0),
            Ship::new(3, 0),
            Ship::new(2, 0),
        ];

        static PLACED_BIT_SHIPS: LazyLock<PlacedBitShips> = LazyLock::new(|| {
            assert!(
                SHIPS.is_sorted_by_key(|ship| ship.index()),
                "SHIPS has to be sorted by ship.index()"
            );
            let placed_ships = SHIPS.map(|ship| {
                let mut placed_ships = [BitBoard::new(Board::new()); 256];
                for dir in [Direction::Horizontal, Direction::Vertical] {
                    for y in 0..SIZE {
                        for x in 0..SIZE {
                            let bit_board_index = dir as usize * 128 + (y * 10 + x);
                            let mut board = Board::new();
                            board.const_place_ship(x, y, dir, ship);

                            placed_ships[bit_board_index] = BitBoard::new(board);
                        }
                    }
                }
                placed_ships
            });
            PlacedBitShips { placed_ships }
        });
        &PLACED_BIT_SHIPS
    }
}
