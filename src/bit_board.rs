use std::sync::LazyLock;
#[allow(unused)]
use std::{
    arch::x86_64::_mm256_popcnt_epi64,
    mem::transmute,
    simd::{cmp::SimdPartialOrd, simd_swizzle, u64x2, u64x4},
};

use super::Board;
#[allow(unused)]
use crate::{
    SIZE,
    board::{Cell, PLACED_SHIPS},
    ship::Ship,
};

pub static PLACED_BITS_SHIPS: LazyLock<Box<[[BitBoard; 256]; 4]>> = LazyLock::new(|| {
    let mut placed_ships = [[BitBoard::new(Board::new()); 256]; 4];
    for ship_i in 0..PLACED_SHIPS.len() {
        for dir in 0..2 {
            for i in 0..128 {
                let bit_board_offset = BitBoard::map_index_to_bit_index(i);
                let bit_board_index = dir * 128 + bit_board_offset;

                let index = dir * 128 + i;
                if bit_board_index < 256 {
                    placed_ships[ship_i][bit_board_index] =
                        BitBoard::new(PLACED_SHIPS[ship_i][index]);
                }
            }
        }
    }
    Box::new(placed_ships)
});
// pub static PLACED_BITS_SHIPS: [[BitBoard; 256]; 10] = {
//     let mut placed_ships = [[BitBoard::new(Board::new()); 256]; 10];
//     let mut ship_i = 0;
//     while ship_i < PLACED_SHIPS.len() {
//         let mut dir = 0;
//         while dir < 2 {
//             let mut i = 0;
//             while i < 256 {
//                 let bit_board_offset = BitBoard::map_index_to_bit_index(i);
//                 let bit_board_index = dir * 128 + bit_board_offset;

//                 let index = dir * 128 + i;
//                 if bit_board_index < 256 {
//                     placed_ships[ship_i][bit_board_index] =
//                         BitBoard::new(PLACED_SHIPS[ship_i][index]);
//                 }
//                 i += 1;
//             }
//             dir += 1;
//         }
//         ship_i += 1;
//     }
//     placed_ships
// };

#[derive(Clone, Copy)]
#[repr(align(32))]
pub struct BitBoard {
    // pub protected: u64x2,
    // pub ship: u64x2,
    pub protected_and_ship: u64x4,
}
impl BitBoard {
    const ALLOWABLE_BITS: u64x2 = u64x2::from_array([(1 << 40) - 1, (1 << 60) - 1]);
    pub fn protected(&self) -> u64x2 {
        simd_swizzle!(self.protected_and_ship, [0, 1])
    }
    pub fn ship(&self) -> u64x2 {
        simd_swizzle!(self.protected_and_ship, [2, 3])
    }
    pub const fn new(board: Board) -> Self {
        let mut protected = 0u128;
        let mut ship = 0u128;

        let mut i = 0;

        while i < u128::BITS as usize {
            if i >= 40 && i < 60 {
                match board.cells[i] {
                    Cell::Water => {}
                    Cell::Protected => protected |= 1 << i,
                    Cell::Ship | Cell::ShipHit => {
                        ship |= 1 << i;
                        protected |= 1 << i
                    }
                }
            }
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
        let protected = unsafe { transmute::<u128, u64x2>(protected) };
        let ship = unsafe { transmute::<u128, u64x2>(ship) };
        Self {
            protected_and_ship: u64x4::from_array([
                protected.as_array()[0],
                protected.as_array()[1],
                ship.as_array()[0],
                ship.as_array()[1],
            ]),
        }
    }
    pub const fn map_index_to_bit_index(index: usize) -> usize {
        if index < 40 {
            index
        } else {
            (index - 40) + 64 // put it into the next u64 to make further calculations easier
        }
    }
    // #[inline(never)]
    fn place_ship(&mut self, index: usize, ship: Ship) {
        let placed_ship_board = unsafe {
            PLACED_BITS_SHIPS
                .get_unchecked(ship.index)
                .get_unchecked(index)
        };

        self.protected_and_ship |= placed_ship_board.protected_and_ship;
    }
    // #[inline(never)]
    fn allowable_ship_placements<const S: Ship>(&self) -> u64x4 {
        let allowable = !self.protected();
        if S.length() == 1 {
            return simd_swizzle!(
                allowable & Self::ALLOWABLE_BITS,
                u64x2::splat(0),
                [0, 1, 2, 3]
            );
        }

        let allowable = !self.protected();

        let wide_allowable = simd_swizzle!(allowable, [0, 1, 0, 1]);
        let mut total_allowable = wide_allowable;

        for i in 1..S.length().min(3) as u64 {
            // only two steps can be shifted without needing the top bits from the top u64
            total_allowable &= wide_allowable >> u64x4::from_array([i, i, i * 10, i * 10]);
        }

        for i in S.length().min(3) as u64..S.length() as u64 {
            // need to first shift in the low bits from the top u64

            let shifted = wide_allowable >> u64x4::from_array([i, i, i * 10, i * 10]);
            let shifted_in_top_bits = simd_swizzle!(wide_allowable, [0, 1, 3, 3])
                << u64x4::from_array([i, i, ((6 - i) * 10), i * 10]);

            let allowable_mask = shifted | shifted_in_top_bits;

            total_allowable &= allowable_mask;
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
        let y_ship_mask = {
            let low = (1 << (4 * SIZE)) - 1; // group of the lower 4 rows
            let high = (1 << ((7 - S.length()) * SIZE)) - 1; // when the ship length is over 1 the top rows are cut off
            u64x2::from_array([low, high])
        };
        total_allowable &= simd_swizzle!(x_ship_mask, y_ship_mask, [0, 1, 2, 3]);

        total_allowable
    }
    #[inline(never)]
    pub fn random_place_ship<const S: Ship>(&mut self, random_value: u32) {
        // pub fn random_place_ship<const S: Ship>(&mut self, ship: Ship, random_value: u32) {
        let ship_placements = self.allowable_ship_placements::<S>();

        if S.length() == 1 {
            let length = ship_placements[0].count_ones() + ship_placements[1].count_ones();
            unsafe {
                std::hint::assert_unchecked(length != 0);
            }
            let total_index = random_value % length;

            let index =
                nth_set_bit_u64x2(simd_swizzle!(ship_placements, [0, 1]), total_index) as usize;
            self.place_ship(index, S);
            return;
        }

        let ship_counts = if cfg!(any(
            target_feature = "avx512vpopcntdq",
            target_feature = "avx512vl"
        )) {
            let ship_counts: u64x4 = unsafe { _mm256_popcnt_epi64(ship_placements.into()) }.into();
            ship_counts
        } else {
            u64x4::from_array(
                ship_placements
                    .to_array()
                    .map(|val| val.count_ones() as u64),
            )
        };

        let ship_index = if cfg!(any(target_feature = "avx512f")) {
            let ship_counts_shift_1 =
                ship_counts + std::simd::simd_swizzle!(ship_counts, u64x4::splat(0), [4, 0, 1, 2]);
            let ship_counts_all = ship_counts_shift_1
                + std::simd::simd_swizzle!(ship_counts_shift_1, u64x4::splat(0), [4, 4, 0, 1]);

            let sum = ship_counts_all[3] as u32;

            unsafe {
                std::hint::assert_unchecked(sum != 0);
            }
            let total_index = random_value % sum;

            let bit_field_index = ship_counts_all
                .simd_gt(u64x4::splat(total_index as u64))
                .to_bitmask()
                .count_ones();
            let over_total_index = ship_counts_all
                .simd_gt(u64x4::splat(total_index as u64))
                .to_bitmask();

            let shiftet_ship_placements: u64x4 = unsafe {
                std::arch::x86_64::_mm256_maskz_compress_epi64(
                    over_total_index as u8,
                    ship_placements.into(),
                )
            }
            .into();
            let shiftet_ship_counts_all: u64x4 = unsafe {
                std::arch::x86_64::_mm256_maskz_compress_epi64(
                    over_total_index as u8,
                    ship_counts_all.into(),
                )
            }
            .into();
            let bit_index_to_select = total_index - shiftet_ship_counts_all[0] as u32;
            let bit_field = shiftet_ship_placements[0];
            (nth_set_bit_index_u64(bit_field, bit_index_to_select) + bit_field_index * 64) as usize
        } else {
            let x_ships = (ship_counts[0] + ship_counts[1]) as u32;
            let y_ships = (ship_counts[2] + ship_counts[3]) as u32;
            // let x_ships = ship_placements[0].count_ones() + ship_placements[1].count_ones();
            // let y_ships = ship_placements[2].count_ones() + ship_placements[3].count_ones();

            let total_possibilities = x_ships + y_ships;
            unsafe {
                std::hint::assert_unchecked(total_possibilities != 0);
            }
            let total_index = random_value % total_possibilities;

            let (bit_map, index, offset) = if total_index < x_ships {
                (simd_swizzle!(ship_placements, [0, 1]), total_index, 0)
            } else {
                (
                    simd_swizzle!(ship_placements, [2, 3]),
                    total_index - x_ships,
                    128,
                ) // there is a brache for ship_placments because ther are 128 and not 64 but and thus can't be move by cmov
            };
            nth_set_bit_u64x2(bit_map, index) as usize + offset
        };

        self.place_ship(ship_index as usize, S)
    }
}

fn nth_set_bit_index_u64(bit_field: u64, n: u32) -> u32 {
    let spread_bits = unsafe { std::arch::x86_64::_pdep_u64(1 << n, bit_field) };
    spread_bits.trailing_zeros()
}
fn nth_set_bit_u64x2(set_bits: u64x2, valid_ship_index: u32) -> u32 {
    let low_set_bits = set_bits[0].count_ones();

    let (target_n, num, offset) = if valid_ship_index < low_set_bits {
        (valid_ship_index, set_bits[0], 0)
    } else {
        (valid_ship_index - low_set_bits, set_bits[1], 64)
    };
    nth_set_bit_index_u64(num, target_n) + offset
}
