use core::simd::{simd_swizzle, u64x4};
use std::{
    arch::x86_64::{self},
    hint::black_box,
    iter::zip,
    simd::{ToBytes, num::SimdInt, prelude::*},
};
#[allow(unused)]
use std::{
    arch::x86_64::{_mm256_popcnt_epi64, _mm512_popcnt_epi64},
    mem::transmute,
    simd::{Mask, Swizzle, cmp::SimdPartialOrd, num::SimdUint},
};

use crate::{
    bit_iter::BitIter,
    board::Board,
    solver::{PlacedBitShips, SpecialRng},
};
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
        // for i in 0..3 {
        //     unsafe {
        //         *self.protected_and_ship.get_unchecked_mut(i) &=
        //             *placed_ship_board.protected_and_ship.get_unchecked(i);
        //     }
        //     // self.protected_and_ship[i] &= placed_ship_board.protected_and_ship[i];
        // }
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

    pub fn random_place_ship<const S: Ship>(
        &mut self,
        placed_bit_ships: &PlacedBitShips,
        special_rng: &mut SpecialRng,
    ) {
        let ship_placements = self.allowable::<S>();

        let possible_placements_count: u64x4 =
            unsafe { _mm256_popcnt_epi64(ship_placements.into()) }.into();

        if S.length() == 1 {
            let length =
                possible_placements_count + possible_placements_count.shift_elements_left::<1>(0);
            let total_index = special_rng.wide_get_random(length)[0] as u32;

            let index =
                nth_set_bit_u64x2(simd_swizzle!(ship_placements, [0, 1]), total_index) as usize;
            self.place_ship::<S>(index as u8, placed_bit_ships);
            return;
        }

        let ship_index = nth_set_bit_u64x4(ship_placements, possible_placements_count, special_rng);

        self.place_ship::<S>(ship_index as u8, placed_bit_ships);
    }
    pub fn placed_ships_to_board(&self) -> Board {
        let mut board = Board::new();

        println!("{:x}", self.ship()[0]);

        for set_ship in BitIter::new(simd_swizzle!(self.ship(), [0, 1, 0, 1])) {
            if set_ship >= 128 {
                break;
            }
            board.cells[set_ship as usize] = Cell::Ship;
        }
        board
    }
}

fn nth_set_bit_u64(bit_field: u64, n: u32) -> u32 {
    let spread_bits = unsafe { x86_64::_pdep_u64(1u64.wrapping_shl(n), bit_field) };
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
    let running_sum_temp =
        set_bits_counted_ones + set_bits_counted_ones.shift_elements_right::<1>(0);
    let running_sum_all = running_sum_temp + running_sum_temp.shift_elements_right::<2>(0);

    let bit_index_at_3 = special_rng.wide_get_random(running_sum_all);
    let all_bit_index = simd_swizzle!(bit_index_at_3, [3, 3, 3, 3]);

    let running_sum_adjusted = running_sum_all - set_bits_counted_ones;
    let over_bit_index = running_sum_all.simd_le(all_bit_index);

    let over_bit_index = over_bit_index.to_int();
    let u64_index = -(over_bit_index
        + over_bit_index.shift_elements_left::<1>(0)
        + over_bit_index.shift_elements_left::<2>(0));

    let set_bit_front: u64x4 =
        unsafe { x86_64::_mm256_permutexvar_epi64(u64_index.into(), set_bits.into()) }.into();
    let adjusted_running_sum_front: u64x4 =
        unsafe { x86_64::_mm256_permutexvar_epi64(u64_index.into(), running_sum_adjusted.into()) }
            .into();

    u64_index[0] as u32 * 64
        + nth_set_bit_u64(
            set_bit_front[0],
            (all_bit_index[0] - adjusted_running_sum_front[0]) as u32,
        )
}

// #[inline(never)]
fn double_nth_set_bit_u64x4(set_bits: u64x8, special_rng: &mut SpecialRng) -> [u32; 2] {
    let set_bits_counts: u64x8 = unsafe { _mm512_popcnt_epi64(set_bits.into()) }.into();

    let small_set_counts_bits: u16x8 = set_bits_counts.cast();
    let small_set_bits_u64: u64x2 = u64x2::from_ne_bytes(small_set_counts_bits.to_ne_bytes());

    let total_sum: u16x8 =
        unsafe { x86_64::_mm_sad_epu8(small_set_counts_bits.into(), u16x8::splat(0).into()) }
            .into();

    let bit_index_at_0_and_4 = special_rng.get_random_u16(total_sum);

    let running_sum = small_set_bits_u64 * u64x2::splat(0x0001_0001_0001_0001);

    let running_sum: u16x8 = u16x8::from_ne_bytes(running_sum.to_ne_bytes());
    let all_bit_index = simd_swizzle!(bit_index_at_0_and_4, [0, 0, 0, 0, 4, 4, 4, 4]);

    let running_sum_adjusted = running_sum - small_set_counts_bits;
    let over_bitindex_mask = running_sum.simd_le(all_bit_index);

    let over_bitindex_compact: i16x8 = over_bitindex_mask.to_int();

    let single_bit_when_over_bitindex = over_bitindex_compact & i16x8::splat(1);
    let local_u64_index: u64x2 = unsafe {
        x86_64::_mm_sad_epu8(single_bit_when_over_bitindex.into(), u8x16::splat(0).into())
    }
    .into();
    let u64_index = local_u64_index + u64x2::from_array([0, 4]);
    let u64_index_wide = simd_swizzle!(u64_index, u64x2::splat(0), [0, 1, 2, 2, 2, 2, 2, 2,]);
    let u64_index_extra_wide: i64x8 = u64_index_wide.cast();
    let set_bit_front: u64x8 =
        unsafe { x86_64::_mm512_permutexvar_epi64(u64_index_extra_wide.into(), set_bits.into()) }
            .into();
    let adjusted_running_sum_front: u16x8 =
        unsafe { x86_64::_mm_permutexvar_epi16(u64_index.into(), running_sum_adjusted.into()) }
            .into();

    [
        local_u64_index[0] as u32 * 64
            + nth_set_bit_u64(
                set_bit_front[0],
                (all_bit_index[0] - adjusted_running_sum_front[0]) as u32,
            ),
        local_u64_index[1] as u32 * 64
            + nth_set_bit_u64(
                set_bit_front[1],
                (all_bit_index[4] - adjusted_running_sum_front[4]) as u32,
            ),
    ]
}
// #[inline(never)]
fn octa_nth_set_bit_u64x4(
    set_bits: [u64x8; 4],
    set_bits_counted_ones: u16x32,
    special_rng: &mut SpecialRng,
) -> u64x8 {
    let small_set_bits_u64: u64x8 = u64x8::from_ne_bytes(set_bits_counted_ones.to_ne_bytes());

    let total_sum: u16x32 =
        unsafe { x86_64::_mm512_sad_epu8(set_bits_counted_ones.into(), u16x32::splat(0).into()) }
            .into();

    let bit_index_at_0_4_8_12_16_20_24_28 = special_rng.extra_wide_get_random_u16_(total_sum);

    // let running_sum = small_set_bits_u64;
    let running_sum = small_set_bits_u64 * u64x8::splat(0x0001_0001_0001_0001);

    let running_sum: u16x32 = u16x32::from_ne_bytes(running_sum.to_ne_bytes());

    const OFFSETS: [usize; 32] = [
        0, 0, 0, 0, 4, 4, 4, 4, 8, 8, 8, 8, 12, 12, 12, 12, 16, 16, 16, 16, 20, 20, 20, 20, 24, 24,
        24, 24, 28, 28, 28, 28,
    ];
    let all_bit_index = simd_swizzle!(bit_index_at_0_4_8_12_16_20_24_28, OFFSETS);

    let running_sum_adjusted = running_sum - set_bits_counted_ones;
    let over_bitindex_mask = running_sum.simd_le(all_bit_index);

    let over_bitindex_compact: i16x32 = over_bitindex_mask.to_int();

    let single_bit_when_over_bitindex = over_bitindex_compact & i16x32::splat(1);
    let local_u64_index: u64x8 = unsafe {
        x86_64::_mm512_sad_epu8(single_bit_when_over_bitindex.into(), u8x64::splat(0).into())
    }
    .into();
    let u64_index = local_u64_index + u64x8::from_array([0, 4, 8, 12, 16, 20, 24, 28]);

    // let u64_index_extra_wide: i64x8 = u64_index_wide.cast();
    let set_bit_front: [u64x8; 4] = unsafe {
        [
            x86_64::_mm512_permutexvar_epi64(u64_index.into(), set_bits[0].into()).into(),
            x86_64::_mm512_permutexvar_epi64(u64_index.into(), set_bits[1].into()).into(),
            x86_64::_mm512_permutexvar_epi64(u64_index.into(), set_bits[2].into()).into(),
            x86_64::_mm512_permutexvar_epi64(u64_index.into(), set_bits[3].into()).into(),
        ]
    };
    let adjusted_running_sum_front: u16x32 =
        unsafe { x86_64::_mm512_permutexvar_epi16(u64_index.into(), running_sum_adjusted.into()) }
            .into();

    u64x8::from_array([
        local_u64_index[0] as u64 * 64
            + nth_set_bit_u64(
                set_bit_front[0][0],
                (all_bit_index[0] - adjusted_running_sum_front[0]) as u32,
            ) as u64,
        local_u64_index[1] as u64 * 64
            + nth_set_bit_u64(
                set_bit_front[0][1],
                (all_bit_index[4] - adjusted_running_sum_front[4]) as u32,
            ) as u64,
        local_u64_index[2] as u64 * 64
            + nth_set_bit_u64(
                set_bit_front[1][2],
                (all_bit_index[8] - adjusted_running_sum_front[8]) as u32,
            ) as u64,
        local_u64_index[3] as u64 * 64
            + nth_set_bit_u64(
                set_bit_front[1][3],
                (all_bit_index[12] - adjusted_running_sum_front[12]) as u32,
            ) as u64,
        local_u64_index[4] as u64 * 64
            + nth_set_bit_u64(
                set_bit_front[2][4],
                (all_bit_index[16] - adjusted_running_sum_front[16]) as u32,
            ) as u64,
        local_u64_index[5] as u64 * 64
            + nth_set_bit_u64(
                set_bit_front[2][5],
                (all_bit_index[20] - adjusted_running_sum_front[20]) as u32,
            ) as u64,
        local_u64_index[6] as u64 * 64
            + nth_set_bit_u64(
                set_bit_front[3][6],
                (all_bit_index[24] - adjusted_running_sum_front[24]) as u32,
            ) as u64,
        local_u64_index[7] as u64 * 64
            + nth_set_bit_u64(
                set_bit_front[3][7],
                (all_bit_index[28] - adjusted_running_sum_front[28]) as u32,
            ) as u64,
    ])
}

fn nth_set_bit_u64x8(set_bits: u64x8, indecies: u16x32) -> u64x8 {
    // let indecies: u64x8 = u64x8::from_ne_bytes(indecies.to_ne_bytes());

    black_box(indecies);
    black_box(set_bits);
    // black_box(set_bits + (indecies << 2));
    // black_box(set_bits + (indecies << 2));
    // black_box(set_bits + (indecies << 2));
    // black_box(set_bits + (indecies << 2));
    // black_box(set_bits + (indecies << 2));
    // black_box(set_bits + (indecies << 2));
    // black_box(set_bits + (indecies << 2));
    // black_box(set_bits + (indecies << 2));

    black_box(u64x8::splat(0))
}
