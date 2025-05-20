use core::simd::u64x4;
use std::cmp::Reverse;

use crate::ship::Ship;
use crate::{
    SIZE,
    board::{Board, Direction},
    solver::{SHIP_COUNT, SHIPS},
};

pub const fn count_ships_to_place(should_place_ship: [bool; SHIP_COUNT]) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < SHIP_COUNT {
        if should_place_ship[i] {
            count += 1;
        }
        i += 1;
    }
    count
}
#[derive(Clone, Copy)]
#[repr(align(256))]
pub struct BitBoard<const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT]>
where
    [(); count_ships_to_place(SHOULD_PLACE_SHIP)]:,
{
    // pub protected_and_ship: [u64x8; 3],
    protected: [u64x4; count_ships_to_place(SHOULD_PLACE_SHIP)],
}
impl<const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT]> BitBoard<SHOULD_PLACE_SHIP>
where
    [(); count_ships_to_place(SHOULD_PLACE_SHIP)]:,
{
    const SHIP_COUNT: usize = count_ships_to_place(SHOULD_PLACE_SHIP);
    pub fn allowable<const SHIP_INDEX: usize>(&self) -> u64x4 {
        let mut count = 0;
        for i in 0..SHIP_INDEX {
            if SHOULD_PLACE_SHIP[i] {
                count += 1;
            }
        }
        // println!("count: {count}, index: {SHIP_INDEX}, should_place: {SHOULD_PLACE_SHIP:?}");
        self.protected[count]

        // match Ship::from_index(SHIP_INDEX).length() {
        //     1 => simd_swizzle!(self.protected_and_ship[0], [2, 3, 2, 3]),
        //     2 => simd_swizzle!(self.protected_and_ship[0], [4, 5, 6, 7]),
        //     3 => simd_swizzle!(self.protected_and_ship[1], [0, 1, 2, 3]),
        //     4 => simd_swizzle!(self.protected_and_ship[1], [4, 5, 6, 7]),
        //     5 => simd_swizzle!(self.protected_and_ship[2], [0, 1, 2, 3]),
        //     6 => simd_swizzle!(self.protected_and_ship[2], [4, 5, 6, 7]),
        //     _ => unreachable!("Invalid ship length"),
        // }
    }
    pub fn new(board: Board) -> Self {
        let protected = board.to_protected();

        let pro_2 = protected.shifted_protected::<{ Ship::new(2) }>();
        let pro_3 = protected.shifted_protected::<{ Ship::new(3) }>();
        let pro_4 = protected.shifted_protected::<{ Ship::new(4) }>();
        let pro_5 = protected.shifted_protected::<{ Ship::new(5) }>();

        let protected: Vec<_> = SHOULD_PLACE_SHIP
            .iter()
            .zip([pro_5, pro_4, pro_3, pro_3, pro_2])
            .filter(|(should_place, _)| **should_place)
            .map(|(_, protected)| protected)
            .collect();

        Self {
            // protected_and_ship: [
            //     simd_swizzle!(
            //         u64x4::from_array([ship[0], ship[1], protected_1[0], protected_1[1]]),
            //         protected_2,
            //         [0, 1, 2, 3, 4, 5, 6, 7]
            //     ),
            //     simd_swizzle!(protected_3, protected_4, [0, 1, 2, 3, 4, 5, 6, 7]),
            //     simd_swizzle!(protected_5, protected_6, [0, 1, 2, 3, 4, 5, 6, 7]),
            // ],
            protected: protected.try_into().unwrap(),
        }
    }
    #[must_use]
    pub fn place_ship<const SHIP_INDEX: usize>(
        self,
        index: u8,
        placed_bit_ships: &PlacedBitShips<SHOULD_PLACE_SHIP>,
    ) -> Self {
        let mut this = self;
        let placed_ship_board = unsafe {
            placed_bit_ships
                .placed_ships
                .get_unchecked(SHIP_INDEX)
                .get_unchecked(index as usize)
        };

        // we only need the ships that are shorter than the current ship
        let remaining_ships = const { ship_count_from_ship_index(SHIP_INDEX, SHOULD_PLACE_SHIP) };

        for i in remaining_ships..this.protected.len() {
            this.protected[i] &= placed_ship_board.protected[i];
        }

        // we only need the ships that are shorter than the current ship
        // for i in 0..Ship::from_index(SHIP_INDEX).length().div_ceil(2) {
        //     this.protected_and_ship[i] &= placed_ship_board.protected_and_ship[i];
        // }
        this
    }
}

const fn ship_count_from_ship_index(
    ship_index: usize,
    should_place_ship: [bool; SHIP_COUNT],
) -> usize {
    let mut i = 0;
    let mut ship_count = 0;
    while i < ship_index {
        if should_place_ship[i] {
            ship_count += 1;
        }
        i += 1;
    }
    ship_count
}

pub struct PlacedBitShips<const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT]>
where
    [(); count_ships_to_place(SHOULD_PLACE_SHIP)]:,
{
    pub placed_ships: [[BitBoard<SHOULD_PLACE_SHIP>; 256]; 5],
}
impl<const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT]> PlacedBitShips<SHOULD_PLACE_SHIP>
where
    [(); count_ships_to_place(SHOULD_PLACE_SHIP)]:,
{
    pub fn new() -> Self {
        assert!(
            SHIPS.is_sorted_by_key(|ship| Reverse(ship.length())),
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
    }
}
