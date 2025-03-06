#[allow(unused)]
use std::{
    arch::x86_64::_mm256_popcnt_epi64,
    mem::transmute,
    simd::{
        Mask, Swizzle, cmp::SimdPartialOrd, i32x8, i64x8, mask32x8, simd_swizzle, u64x2, u64x4,
    },
    sync::LazyLock,
};

use super::Board;
use crate::solver::{PlacedBitShips, SpecialRng};
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

#[derive(Clone, Copy)]
#[repr(align(32))]
pub struct BitBoard {
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
                        protected |= 1 << i;
                    }
                }
            }
            let bit_index = Self::map_index_to_bit_index(i);
            match board.cells[i] {
                Cell::Water => {}
                Cell::Protected => protected |= 1 << bit_index,
                Cell::Ship | Cell::ShipHit => {
                    ship |= 1 << bit_index;
                    protected |= 1 << bit_index;
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
    fn place_ship(&mut self, index: usize, ship: Ship, placed_bit_ships: &PlacedBitShips) {
        let placed_ship_board = unsafe {
            placed_bit_ships
                .placed_ships
                .get_unchecked(ship.index())
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
        // let wide_allowable = simd_swizzle!(allowable, [0, 1, 0, 1]);
        // wide_allowable

        let wide_allowable = simd_swizzle!(allowable, [0, 1, 0, 1]);
        let mut total_allowable = wide_allowable;

        for i in 1..S.length() as u64 {
            total_allowable &= shift(wide_allowable, i);
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
    // #[inline(never)]
    pub fn random_place_ship<const S: Ship>(
        &mut self,
        placed_bit_ships: &PlacedBitShips,
        special_rng: &mut SpecialRng,
    ) {
        // pub fn random_place_ship<const S: Ship>(&mut self, ship: Ship, random_value: u32) {
        let ship_placements = self.allowable_ship_placements::<S>();

        if S.length() == 1 {
            let length = ship_placements[0].count_ones() + ship_placements[1].count_ones();
            let total_index = special_rng.get_random(length as u8);

            let index =
                nth_set_bit_u64x2(simd_swizzle!(ship_placements, [0, 1]), total_index) as usize;
            self.place_ship(index, S, placed_bit_ships);
            return;
        }

        let possible_placements_count = u64x4::from_array(
            ship_placements
                .to_array()
                .map(|val| val.count_ones() as u64),
        );

        let x_ships = possible_placements_count[0] + possible_placements_count[1];
        let y_ships = possible_placements_count[2] + possible_placements_count[3];
        let total_possibilities = x_ships + y_ships;

        let total_index = special_rng.get_random(total_possibilities as u8);

        let (bit_map, index, offset) = if (total_index as u64) < x_ships {
            (simd_swizzle!(ship_placements, [0, 1]), total_index, 0)
        } else {
            (
                simd_swizzle!(ship_placements, [2, 3]),
                total_index - x_ships as u32,
                128,
            ) // there is a brache for ship_placments because ther are 128 and not 64 but and thus can't be move by cmov
        };
        let ship_index = nth_set_bit_u64x2(bit_map, index) as usize + offset;

        // let possible_placements_count: u64x4 =
        //     unsafe { _mm256_popcnt_epi64(ship_placements.into()) }.into();
        // let ship_index = nth_set_bit_u64x4(ship_placements, possible_placements_count, special_rng);

        self.place_ship(ship_index as usize, S, placed_bit_ships);
    }
}

fn nth_set_bit_u64(bit_field: u64, n: u32) -> u32 {
    let spread_bits = unsafe { std::arch::x86_64::_pdep_u64(1 << n, bit_field) };
    spread_bits.trailing_zeros()
}
// #[inline(never)]
fn nth_set_bit_u64x2(set_bits: u64x2, valid_ship_index: u32) -> u32 {
    let low_set_bits = set_bits[0].count_ones();

    let (target_n, num, offset) = if valid_ship_index < low_set_bits {
        (valid_ship_index, set_bits[0], 0)
    } else {
        (valid_ship_index - low_set_bits, set_bits[1], 64)
    };
    nth_set_bit_u64(num, target_n) + offset
}
// #[inline(never)]
fn nth_set_bit_u64x4(
    set_bits: u64x4,
    set_bits_counted_ones: u64x4,
    special_rng: &mut SpecialRng,
) -> u32 {
    let running_sum_all = set_bits_counted_ones
        + set_bits_counted_ones.shift_elements_right::<2>(0)
        + set_bits_counted_ones.shift_elements_right::<1>(0)
        + set_bits_counted_ones.shift_elements_right::<3>(0);

    let bit_index_at_3 = special_rng.wide_get_random(running_sum_all);
    let all_bit_index = simd_swizzle!(bit_index_at_3, [3, 3, 3, 3]);

    let running_sum_adjusted = running_sum_all - set_bits_counted_ones;
    let over_bit_index = running_sum_all.simd_le(all_bit_index);

    let over_bit_index = over_bit_index.to_int();
    let u64_index = -(over_bit_index
        + over_bit_index.shift_elements_left::<1>(0)
        + over_bit_index.shift_elements_left::<2>(0));

    let set_bit_front: u64x4 =
        unsafe { std::arch::x86_64::_mm256_permutexvar_epi64(u64_index.into(), set_bits.into()) }
            .into();
    let adjusted_running_sum_front: u64x4 = unsafe {
        std::arch::x86_64::_mm256_permutexvar_epi64(u64_index.into(), running_sum_adjusted.into())
    }
    .into();

    u64_index[0] as u32 * 64
        + nth_set_bit_u64(
            set_bit_front[0],
            (all_bit_index[0] - adjusted_running_sum_front[0]) as u32,
        )
}

fn shift(val: u64x4, shift_amount: u64) -> u64x4 {
    let shift = u64x4::from_array([
        shift_amount,
        shift_amount,
        shift_amount * 10,
        shift_amount * 10,
    ]);
    if shift_amount < 3 {
        // only two steps can be shifted without needing the top bits from the top u64
        val >> shift
    } else {
        let shifted = val >> shift;
        let shifted_in_top_bits = simd_swizzle!(val, [0, 1, 3, 3]) << shift;

        shifted | shifted_in_top_bits
    }
}
