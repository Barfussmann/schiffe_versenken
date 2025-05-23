use core::simd::u64x4;
use std::cmp::Reverse;
use std::iter::zip;

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
#[derive(Debug, Clone, Copy)]
// #[repr(align(256))]
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
        self.protected[count]
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
            protected: protected.try_into().unwrap(),
        }
    }
    #[must_use]
    pub fn place_ship<const SHIP_INDEX: usize>(
        mut self,
        index: u8,
        placed_bit_ships: &PlacedBitShips<SHOULD_PLACE_SHIP>,
    ) -> Self {
        let ship_index = const { ship_count_to_ship_index(SHIP_INDEX, SHOULD_PLACE_SHIP) };
        let placed_ship_board = unsafe {
            placed_bit_ships
                .placed_ships
                .get_unchecked(ship_index)
                .get_unchecked(index as usize)
        };

        // we only need the ships that are shorter than the current ship
        let remaining_ships = const { ship_count_to_ship_index(SHIP_INDEX, SHOULD_PLACE_SHIP) };

        for i in remaining_ships..self.protected.len() {
            self.protected[i] &= placed_ship_board.protected[i];
        }

        self
    }
}

pub const fn ship_count_to_ship_index(
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
    pub placed_ships: [[BitBoard<SHOULD_PLACE_SHIP>; 256]; count_ships_to_place(SHOULD_PLACE_SHIP)],
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
        let placed_ships = zip(SHIPS, SHOULD_PLACE_SHIP)
            .filter(|(_, should_place)| *should_place)
            .map(|(ship, _)| {
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
            })
            .collect::<Vec<_>>();
        PlacedBitShips {
            placed_ships: placed_ships.try_into().unwrap(),
        }
    }
}
