use core::simd::u64x4;

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

        let pro_2 = protected.shifted_protected::<{ Ship::new(2) }>();
        let pro_3 = protected.shifted_protected::<{ Ship::new(3) }>();
        let pro_4 = protected.shifted_protected::<{ Ship::new(4) }>();
        let pro_5 = protected.shifted_protected::<{ Ship::new(5) }>();

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
#[derive(Clone)]
pub struct PlacedBitShips<const N: usize> {
    pub placed_ships: [[BitBoard<N>; 256]; N],
}
impl<const N: usize> PlacedBitShips<N> {
    pub fn new(ship_counts: ShipCounts) -> Self {
        let placed_ships = ship_counts
            .iter_ships()
            .map(|ship| {
                let mut placed_ships = [Board::new().to_bitboard(ship_counts); 256];
                for dir in [Direction::Horizontal, Direction::Vertical] {
                    for y in 0..SIZE {
                        for x in 0..SIZE {
                            let bit_board_index = dir as usize * 128 + (y * 10 + x);
                            let mut board = Board::new();
                            board.const_place_ship(x, y, dir, ship);

                            placed_ships[bit_board_index] = BitBoard::new(&board, ship_counts);
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
