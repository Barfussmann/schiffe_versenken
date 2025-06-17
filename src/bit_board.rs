use core::simd::u64x4;
use std::sync::LazyLock;

use crate::ship::{Ship, ShipCounts};
use crate::{
    SIZE,
    board::{Board, Direction},
};

#[derive(Debug, Clone, Copy)]
pub struct BitBoard<const N: usize> {
    protected: [u64x4; N],
}
impl<const N: usize> BitBoard<N> {
    pub fn allowable<const SHIP_INDEX: usize>(&self) -> u64x4 {
        self.protected[SHIP_INDEX]
    }
    pub fn new(board: &Board, ship_counts: ShipCounts) -> Self {
        let protected = board.to_protected();

        let pro_2 = protected.shift_protected::<{ Ship::new(2) }>();
        let pro_3 = protected.shift_protected::<{ Ship::new(3) }>();
        let pro_4 = protected.shift_protected::<{ Ship::new(4) }>();
        let pro_5 = protected.shift_protected::<{ Ship::new(5) }>();

        let protected: Vec<_> = ship_counts
            .counts()
            .iter()
            .zip([pro_5, pro_4, pro_3, pro_2])
            .flat_map(|(ship_count, ship)| std::iter::repeat_n(ship, *ship_count))
            .collect();

        Self {
            protected: protected.try_into().unwrap(),
        }
    }
    #[must_use]
    pub fn place_ship<const SHIP_INDEX: usize>(
        mut self,
        index: u8,
        placed_bit_ships: &PlacedBitShips<N>,
    ) -> Self {
        let placed_ship_board = unsafe {
            placed_bit_ships
                .placed_ships
                .get_unchecked(SHIP_INDEX)
                .get_unchecked(index as usize)
        };

        // we only need the ships that are shorter than the current ship

        for i in SHIP_INDEX + 1..self.protected.len() {
            self.protected[i] &= placed_ship_board.protected[i];
        }

        self
    }
}
pub struct PlacedBitShips<const N: usize> {
    pub placed_ships: [[BitBoard<N>; 256]; N],
}
impl PlacedBitShips<0> {
    pub fn new(ship_counts: ShipCounts) -> &'static Self {
        static LOOKUP: LazyLock<Vec<(ShipCounts, PlacedBitShips<0>)>> =
            LazyLock::new(PlacedBitShips::gen_lookup);
        &LOOKUP
            .iter()
            .find(|(cached_ship_count, _)| *cached_ship_count == ship_counts)
            .unwrap()
            .1
    }
}
impl PlacedBitShips<1> {
    pub fn new(ship_counts: ShipCounts) -> &'static Self {
        static LOOKUP: LazyLock<Vec<(ShipCounts, PlacedBitShips<1>)>> =
            LazyLock::new(PlacedBitShips::gen_lookup);
        &LOOKUP
            .iter()
            .find(|(cached_ship_count, _)| *cached_ship_count == ship_counts)
            .unwrap()
            .1
    }
}
impl PlacedBitShips<2> {
    pub fn new(ship_counts: ShipCounts) -> &'static Self {
        static LOOKUP: LazyLock<Vec<(ShipCounts, PlacedBitShips<2>)>> =
            LazyLock::new(PlacedBitShips::gen_lookup);
        &LOOKUP
            .iter()
            .find(|(cached_ship_count, _)| *cached_ship_count == ship_counts)
            .unwrap()
            .1
    }
}
impl PlacedBitShips<3> {
    pub fn new(ship_counts: ShipCounts) -> &'static Self {
        static LOOKUP: LazyLock<Vec<(ShipCounts, PlacedBitShips<3>)>> =
            LazyLock::new(PlacedBitShips::gen_lookup);
        &LOOKUP
            .iter()
            .find(|(cached_ship_count, _)| *cached_ship_count == ship_counts)
            .unwrap()
            .1
    }
}
impl PlacedBitShips<4> {
    pub fn new(ship_counts: ShipCounts) -> &'static Self {
        static LOOKUP: LazyLock<Vec<(ShipCounts, PlacedBitShips<4>)>> =
            LazyLock::new(PlacedBitShips::gen_lookup);
        &LOOKUP
            .iter()
            .find(|(cached_ship_count, _)| *cached_ship_count == ship_counts)
            .unwrap()
            .1
    }
}
impl PlacedBitShips<5> {
    pub fn new(ship_counts: ShipCounts) -> &'static Self {
        static LOOKUP: LazyLock<Vec<(ShipCounts, PlacedBitShips<5>)>> =
            LazyLock::new(PlacedBitShips::gen_lookup);
        &LOOKUP
            .iter()
            .find(|(cached_ship_count, _)| *cached_ship_count == ship_counts)
            .unwrap()
            .1
    }
}

impl<const N: usize> PlacedBitShips<N> {
    // #[rustfmt::skip]
    // pub fn new(ship_counts: ShipCounts) -> Self {
    //     Self::gen_self(ship_counts)
    // }
    //
    fn gen_lookup() -> Vec<(ShipCounts, Self)> {
        ShipCounts::all_possible_ship_counts()
            .filter(|ship_counts| ship_counts.total_ships() == N)
            .map(|ship_counts| (ship_counts, Self::gen_self(ship_counts)))
            .collect()
    }
    fn gen_self(ship_counts: ShipCounts) -> Self {
        Self {
            placed_ships: ship_counts
                .iter_ships()
                .map(|ship| {
                    let mut placed_ships = [Board::new().to_bitboard(ship_counts); 256];
                    for dir in [Direction::Horizontal, Direction::Vertical] {
                        for y in 0..SIZE {
                            for x in 0..SIZE {
                                let bit_board_index = dir as usize * 128 + (y * 10 + x);
                                let mut board = Board::new();
                                board.place_ship(x, y, dir, ship);

                                placed_ships[bit_board_index] = BitBoard::new(&board, ship_counts);
                            }
                        }
                    }
                    placed_ships
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }
}
