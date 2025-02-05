use std::{
    mem::transmute,
    simd::{u64x2, u64x4},
};

use super::Board;
use crate::{
    SIZE,
    board::{Cell, PLACED_SHIPS},
    ship::Ship,
};

pub static PLACED_BITS_SHIPS: [[BitBoard; 256]; 10] = {
    let mut placed_ships = [[BitBoard::new(Board::new()); 256]; 10];

    let mut ship_i = 0;
    while ship_i < 10 {
        let mut i = 0;
        while i < 256 {
            placed_ships[ship_i][i] = BitBoard::new(PLACED_SHIPS[ship_i][i]);
            i += 1;
        }
        ship_i += 1;
    }

    placed_ships
};

#[derive(Clone, Copy)]
#[repr(align(32))]
pub struct BitBoard {
    pub protected: u64x2,
    pub ship: u64x2,
}
impl BitBoard {
    pub const fn new(board: Board) -> Self {
        let mut protected = 0u128;
        let mut ship = 0u128;

        let mut i = 0;

        while i < u128::BITS as usize {
            let bit_index = Self::map_index_to_bit_index(i);
            match board.cells[i] {
                Cell::Water => {}
                Cell::Protected => protected |= 1 << bit_index,
                Cell::Ship | Cell::ShipHit => {
                    ship |= 1 << bit_index;
                    protected |= 1 << bit_index
                }
            }

            i += 1;
        }
        Self {
            protected: unsafe { transmute::<u128, u64x2>(protected) },
            ship: unsafe { transmute::<u128, u64x2>(ship) },
        }
    }
    const fn map_index_to_bit_index(index: usize) -> usize {
        if index < 40 {
            index
        } else {
            (index - 40) + 64 // put it in to the next u64 to make further calculations easier
        }
    }
    // #[inline(never)]
    fn place_ship(&mut self, index: usize, ship: Ship) {
        let placed_ship_board = unsafe {
            PLACED_BITS_SHIPS
                .get_unchecked(ship.index)
                // .as_flattened()
                // .as_flattened()
                .get_unchecked(index)
        };

        self.protected |= placed_ship_board.protected;
        self.ship |= placed_ship_board.ship;
    }
    // #[inline(never)]
    fn allowable_ship_placements<const S: Ship>(&self) -> (u64x2, u64x2) {
        let allowable = !self.protected;

        let mut ship_placements_x = allowable;
        for i in 1..S.length() as u64 {
            ship_placements_x &= allowable >> i;
        }

        let x_ship_mask = {
            let mut mask = 0u128;
            let single_row_allowable = (1 << (SIZE - (S.length() - 1))) - 1;

            for y in 0..SIZE {
                let bit_index = BitBoard::map_index_to_bit_index(y * SIZE);

                mask |= single_row_allowable << bit_index;
            }

            unsafe { transmute::<u128, u64x2>(mask) }
        };
        ship_placements_x &= x_ship_mask;

        let mut ship_placements_y = allowable;
        for i in 1..S.length() {
            let high = allowable[1];
            let high_shifted_by_one = high >> SIZE;

            let low_shifted_by_one = (allowable[0] >> SIZE) & ((1 << 30) - 1);
            let low_with_high = low_shifted_by_one | (high << 30);

            ship_placements_y &=
                u64x2::from_array([low_with_high, high_shifted_by_one]) >> ((i - 1) * SIZE) as u64;
        }
        let y_ship_mask = {
            let low = (1 << (4 * SIZE)) - 1; // group of the lower 4 rows
            let high = (1 << ((7 - S.length()) * SIZE)) - 1; // when the ship length is over 1 the top rows are cut off
            u64x2::from_array([low, high])
        };
        ship_placements_y &= y_ship_mask;

        (ship_placements_x, ship_placements_y)
    }
    #[inline(never)]
    pub fn random_place_ship<const S: Ship>(&mut self, random_value: u32) {
        // pub fn random_place_ship<const S: Ship>(&mut self, ship: Ship, random_value: u32) {
        let (ship_placements_x, ship_placements_y) = self.allowable_ship_placements::<S>();

        let ship_placements = u64x4::from_array([
            ship_placements_x[0],
            ship_placements_x[1],
            ship_placements_y[0],
            ship_placements_y[1],
        ]);
        let ship_counts = u64x4::from_array(
            ship_placements
                .as_array()
                .map(|placement| placement.count_ones() as u64),
        );

        // let mut running_sum = u64x4::splat(0);

        // let mut sum = 0;
        // for i in 0..4 {
        //     running_sum[i] = sum;
        //     sum += ship_counts[i];
        // }

        // let total_index = random_value % sum as u32;

        // let shifted_running_sum = running_sum + ship_counts;
        // let extract_index = shifted_running_sum
        //     .as_array()
        //     .iter()
        //     .filter(|count| **count < total_index as u64)
        //     .count();

        // unsafe {
        //     std::hint::assert_unchecked(extract_index < 4);
        // }

        // let bit_map = ship_placements[extract_index];
        // let skipped_bits = running_sum[extract_index];

        // let offset = match extract_index {
        //     0 => 0,
        //     1 => 40,
        //     2 => 128,
        //     3 => 128 + 40,
        //     _ => unsafe { std::hint::unreachable_unchecked() },
        // };

        // let index = nth_set_bit_index_u64(bit_map, total_index - skipped_bits as u32) + offset;

        let x_ships = (ship_counts[0] + ship_counts[1]) as u32;
        let y_ships = (ship_counts[2] + ship_counts[3]) as u32;

        // let x_ships = ship_placements_x[0].count_ones() + ship_placements_x[1].count_ones();
        // let y_ships = ship_placements_y[0].count_ones() + ship_placements_y[1].count_ones();

        let total_index = random_value % (x_ships + y_ships);
        // let total_index = rng.u8(..(x_ships + y_ships) as u8) as u32;

        let (bit_map, index, offset) = if total_index < x_ships {
            (ship_placements_x, total_index, 0)
        } else {
            (ship_placements_y, total_index - x_ships, 128) // there is a brache for ship_placments because ther are 128 and not 64 but and thus can't be move by cmov
        };
        let index = set_bit_to_index(bit_map, index) as usize + offset;

        self.place_ship(index as usize, S)
    }
}

fn nth_set_bit_index_u64(num: u64, n: u32) -> u32 {
    let spread_bits = unsafe { std::arch::x86_64::_pdep_u64(1 << n, num) };
    spread_bits.trailing_zeros()
}
fn set_bit_to_index(set_bits: u64x2, valid_ship_index: u32) -> u32 {
    let low_set_bits = set_bits[0].count_ones();

    let (target_n, num, offset) = if valid_ship_index < low_set_bits {
        (valid_ship_index, set_bits[0], 0)
    } else {
        (valid_ship_index - low_set_bits, set_bits[1], 40)
    };
    nth_set_bit_index_u64(num, target_n) + offset

    // 'outer: {
    //     let mut count = 0;
    //     for i in 0..128 {
    //         if num & (1 << i) != 0 {
    //             if count == n {
    //                 break 'outer i;
    //             }
    //             count += 1;
    //         }
    //     }
    //     0
    // };
}
