use crate::{
    bit_iter::ChunkedBitIter,
    board::{Board, Cell},
    ship::Ship,
};

use super::BOARD_SIZE;
use super::SIZE;
use core::{
    convert::TryInto,
    simd::{u64x4, u64x64},
};
use std::{
    fmt::{Display, Write},
    iter::zip,
    ops::Range,
    simd::prelude::*,
};

#[derive(Debug, Clone)]
pub struct ShipPositionCounts {
    counts: [u64x64; 4],
    added_boards: u64,
    small_counts: [u8x64; 4],
    bit_counts: [u64x4; 8],
    bit_counts_total: [u64x4; 32],
    added_bit_fields: usize,
    bit_fields_to_sum: [u64x4; 256],
}
impl ShipPositionCounts {
    pub fn new() -> ShipPositionCounts {
        ShipPositionCounts {
            counts: [u64x64::splat(0); 4],
            added_boards: 0,
            small_counts: [u8x64::splat(0); 4],
            bit_counts: [u64x4::splat(0); 8],
            bit_counts_total: [u64x4::splat(0); 32],
            added_bit_fields: 0,
            bit_fields_to_sum: [u64x4::splat(0); 256],
        }
    }
    fn counts(&self) -> [u64; 256] {
        // let mut counts = [0; 256];
        // for i in 0..256 {
        //     counts[i] = self.counts[i].reduce_sum() + self.counts_bits[i / 64][i % 64];
        // }
        // counts
        self.counts
            .map(|x| x.to_array())
            .as_flattened()
            .try_into()
            .unwrap()
    }
    pub fn add_single_ship(&mut self, index: u8, counts: u64) {
        // self.counts[index as usize] += counts;
        let u64_index = index as usize / 64;
        let rem_index = index as usize % 64;
        self.counts[u64_index][rem_index] += counts;
    }
    #[rustfmt::skip]
    pub fn sum_single_bits(&mut self) {

        // unsafe {
        //     self.bit_fields_to_sum.get_unchecked_mut(self.added_bit_fields..self.added_bit_fields.next_multiple_of(8)).fill(u64x4::splat(0));
        // }
        // self.bit_fields_to_sum[self.added_bit_fields..self.added_bit_fields.next_multiple_of(8)].fill(u64x4::splat(0));
        // self.bit_fields_to_sum[self.added_bit_fields..self.added_bit_fields.next_multiple_of(8)].fill(u64x4::splat(0));
        for i in self.added_bit_fields..self.added_bit_fields.next_multiple_of(8) {
            unsafe {
                *self.bit_fields_to_sum.get_unchecked_mut(i) = u64x4::splat(0);
            }
        }
        for chunk in self.bit_fields_to_sum.array_chunks::<8>().take(self.added_bit_fields.div_ceil(8)) {

            let a_3 = bit_adder(bit_adder([chunk[0]], [chunk[1]]), bit_adder([chunk[2]], [chunk[3]]));
            let b_3 = bit_adder(bit_adder([chunk[4]], [chunk[5]]), bit_adder([chunk[6]], [chunk[7]]));

            let mut a_4 = bit_adder(a_3, b_3);

            bit_adder_in_place(&mut self.bit_counts, &mut a_4);
            // let bit_counts = self.bit_counts;
            // self.bit_counts.copy_from_slice(&bit_adder([a_4[0], a_4[1], a_4[2], a_4[3], u64x4::splat(0), u64x4::splat(0), u64x4::splat(0), u64x4::splat(0)], bit_counts)[..8]);
        }
        // for i in 0..self.added_bit_fields {
        //     let mut carry = self.bit_fields_to_sum[i];
        //     for bit_index in 0..8 {
        //         let next_carry = self.bit_counts[bit_index] & carry;
        //         self.bit_counts[bit_index] ^= carry;
        //         carry = next_carry;
        //     }
        // }
        self.added_bit_fields = 0;
    }
    // #[inline(never)]
    pub fn sum_bit_counts_to_total_bit_counts(&mut self) {
        bit_adder_in_place(&mut self.bit_counts_total, &mut self.bit_counts);
    }
    #[inline(never)]
    pub fn sum_bit_counts(&mut self) {
        for bit_index in 0..8 {
            let mul = 2usize.pow(bit_index as u32);
            for i in 0..4 {
                self.counts[i] -= mask8x64::from_bitmask(self.bit_counts[bit_index][i])
                    .to_int()
                    .cast()
                    * u64x64::splat(mul as u64);
            }
            self.bit_counts[bit_index] = u64x4::splat(0);
        }
        for bit_index in 0..self.bit_counts_total.len() {
            let mul = 2usize.pow(bit_index as u32);
            for i in 0..4 {
                self.counts[i] -= mask8x64::from_bitmask(self.bit_counts_total[bit_index][i])
                    .to_int()
                    .cast()
                    * u64x64::splat(mul as u64);
            }
            self.bit_counts_total[bit_index] = u64x4::splat(0);
        }
    }
    // returns the count of the added ships
    pub fn add_possible_ship_positions_chunked(
        &mut self,
        ship_positions: [u64x4; ChunkedBitIter::CHUNK_SIZE],
    ) -> [u64; ChunkedBitIter::CHUNK_SIZE] {
        let sum_3_a = bit_adder(
            bit_adder([ship_positions[0]], [ship_positions[1]]),
            bit_adder([ship_positions[2]], [ship_positions[3]]),
        );
        let sum_3_b = bit_adder(
            bit_adder([ship_positions[0]], [ship_positions[1]]),
            bit_adder([ship_positions[2]], [ship_positions[3]]),
        );
        let sum_4 = bit_adder(sum_3_a, sum_3_b);

        let bit_counts = self.bit_counts;
        self.bit_counts.copy_from_slice(
            &bit_adder(
                [
                    sum_4[0],
                    sum_4[1],
                    sum_4[2],
                    sum_4[3],
                    u64x4::splat(0),
                    u64x4::splat(0),
                    u64x4::splat(0),
                    u64x4::splat(0),
                ],
                bit_counts,
            )[..8],
        );

        let mut added_ships = [0; ChunkedBitIter::CHUNK_SIZE];
        for chunk_i in 0..ChunkedBitIter::CHUNK_SIZE {
            for i in 0..4 {
                // added_ships[chunk_i] += 17;
                added_ships[chunk_i] += ship_positions[chunk_i][i].count_ones() as u64;
            }
        }

        added_ships
    }
    // returns the count of the added ships
    pub fn add_possible_ship_positions(&mut self, ship_positions: u64x4) -> u64 {
        unsafe {
            *self
                .bit_fields_to_sum
                .get_unchecked_mut(self.added_bit_fields) = ship_positions;
        }
        self.added_bit_fields += 1;

        // let mut carry = ship_positions;
        // for i in 0..8 {
        //     let next_carry = self.bit_counts[i] & carry;
        //     self.bit_counts[i] ^= carry;
        //     carry = next_carry;
        // }

        let mut added_ships = 0;
        for i in 0..4 {
            // added_ships += 17;
            added_ships += ship_positions[i].count_ones() as u64;
            // dbg!(ship_positions[i].count_ones() as u64);
        }
        added_ships
        // u64x4::from_array([
        //     ship_positions[0].count_ones() as u64,
        //     ship_positions[1].count_ones() as u64,
        //     ship_positions[2].count_ones() as u64,
        //     ship_positions[3].count_ones() as u64,
        // ])
    }
}

fn bit_adder_in_place<const N: usize, const M: usize>(vals: &mut [u64x4; N], b: &mut [u64x4; M]) {
    let mut carry = u64x4::splat(0);
    assert!(M <= N);
    for i in 0..M {
        let a = vals[i];
        vals[i] = a ^ b[i] ^ carry;
        carry = (a & b[i]) | (carry & (a ^ b[i]));
        b[i] = u64x4::splat(0);
    }
    for i in M..N {
        let a = vals[i];
        vals[i] = a ^ carry;
        carry &= a
    }
}
fn bit_adder<const N: usize>(a: [u64x4; N], b: [u64x4; N]) -> [u64x4; N + 1] {
    let mut res = [u64x4::splat(0); N + 1];
    let mut carry = u64x4::splat(0);

    for i in 0..N {
        res[i] = a[i] ^ b[i] ^ carry;
        carry = (a[i] & b[i]) | (carry & (a[i] ^ b[i]))
    }
    res[N] = carry;
    res
}

#[derive(Debug, Clone)]
pub struct BoardCounts {
    pub counts: [u64; BOARD_SIZE],
    pub board_count: u64,
}

impl BoardCounts {
    pub fn new() -> BoardCounts {
        BoardCounts {
            counts: [0; BOARD_SIZE],
            board_count: 0,
        }
    }
    pub fn add_board(&mut self, board: Board) {
        for (count, cell) in zip(&mut self.counts, &board.cells) {
            match cell {
                Cell::Ship => {
                    *count += 1;
                }
                Cell::Protected | Cell::Water | Cell::ShipHit => {}
            }
        }
        self.board_count += 1;
    }
    #[inline(never)]
    pub fn add_ship_positions_counts<const SHIP: Ship>(
        &mut self,
        ship_counts: &mut ShipPositionCounts,
    ) {
        ship_counts.sum_bit_counts();
        let counts = ship_counts.counts();
        for i in 0..100 {
            let ship_count_x = counts[i];
            let ship_count_y = counts[i + 128];
            for ship_i in 0..SHIP.length() {
                self.counts[i + ship_i] += ship_count_x;
                if i + ship_i * 10 < 128 {
                    self.counts[i + ship_i * 10] += ship_count_y;
                }
            }
        }
        *ship_counts = ShipPositionCounts::new();
    }
    pub fn add_other_count(&mut self, other: Self) {
        for (self_count, other_count) in zip(&mut self.counts, &other.counts) {
            *self_count += *other_count;
        }
        self.board_count += other.board_count;
    }
}

impl Display for BoardCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        for row in self.counts.chunks(SIZE).take(SIZE) {
            for count in row {
                let probability = (*count as f64) / (self.board_count as f64);

                f.write_fmt(format_args!("{:4.1} ", probability * 100.))?;
                // f.write_fmt(format_args!("{:3.1} ", probability * 100.))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
