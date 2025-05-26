use colored::Colorize;
use colorgrad::Gradient;
use crunchy::unroll;

use crate::{
    board::{Board, Cell},
    ship::ShipCounts,
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
    simd::prelude::*,
};

#[derive(Debug, Clone)]
pub struct ShipCellCounts<const N: usize> {
    pub counts_per_ship_position: [CellCount; N],
}
impl<const N: usize> ShipCellCounts<N> {
    pub fn new() -> Self {
        Self {
            counts_per_ship_position: std::array::from_fn(|_| CellCount::new()),
        }
    }
    #[inline(never)]
    pub fn sum_last_ship(&mut self) {
        self.counts_per_ship_position
            .last_mut()
            .unwrap()
            .sum_single_bits();
    }
}

#[derive(Debug, Clone)]
pub struct CellCount {
    position_counts: [u64x64; 4],
    bit_counts: [u64x4; 8],
    bit_counts_total: [u64x4; 32],
    added_bit_fields: usize,
    bit_fields_to_sum: [u64x4; 512],
}
impl CellCount {
    pub fn new() -> CellCount {
        CellCount {
            position_counts: [u64x64::splat(0); 4],
            bit_counts: [u64x4::splat(0); 8],
            bit_counts_total: [u64x4::splat(0); 32],
            added_bit_fields: 0,
            bit_fields_to_sum: [u64x4::splat(0); 512],
        }
    }
    fn counts(&self) -> [u64; 256] {
        self.position_counts
            .map(|x| x.to_array())
            .as_flattened()
            .try_into()
            .unwrap()
    }
    pub fn add_single_ship(&mut self, index: u8, counts: u64) {
        // self.counts[index as usize] += counts;
        let u64_index = index as usize / 64;
        let rem_index = index as usize % 64;
        self.position_counts[u64_index][rem_index] += counts;
    }
    #[inline(never)]
    pub fn sum_single_bits(&mut self) {
        const BIT_FIELD_CHUNKS: usize = 255;
        // const BIT_FIELD_CHUNKS: usize = 63;
        if self.added_bit_fields < BIT_FIELD_CHUNKS {
            return;
        }
        let start_index = self.added_bit_fields - BIT_FIELD_CHUNKS;
        let bit_fields_to_sum: &[u64x4; BIT_FIELD_CHUNKS] = self.bit_fields_to_sum
            [start_index..self.added_bit_fields]
            .first_chunk::<BIT_FIELD_CHUNKS>()
            .unwrap();

        // let mut sum_low = [u64x4::splat(0); 6];

        // sum_test(bit_fields_to_sum, &mut sum_low);
        // black_box(sum_low);
        // let sum = bit_sum_test_2(bit_fields_to_sum);
        // black_box(sum);

        // sum_test(bit_fields_to_sum, &mut sum_low);
        // bit_adder_in_place(&mut self.bit_counts_total, &sum_low);

        let sum = bit_sum_test_3(bit_fields_to_sum);
        bit_adder_in_place(&mut self.bit_counts_total, &sum);

        self.added_bit_fields -= BIT_FIELD_CHUNKS;
    }
    // #[inline(never)]
    pub fn sum_bit_counts_to_total_bit_counts(&mut self) {
        bit_adder_in_place(&mut self.bit_counts_total, &self.bit_counts);
    }
    #[inline(never)]
    pub fn sum_bit_counts(&mut self) {
        for bit_index in 0..self.bit_counts.len() {
            let mul = 2usize.pow(bit_index as u32);
            for i in 0..4 {
                self.position_counts[i] -= mask8x64::from_bitmask(self.bit_counts[bit_index][i])
                    .to_int()
                    .cast()
                    * u64x64::splat(mul as u64);
            }
            self.bit_counts[bit_index] = u64x4::splat(0);
        }
        for bit_index in 0..self.bit_counts_total.len() {
            let mul = 2usize.pow(bit_index as u32);
            for i in 0..4 {
                self.position_counts[i] -=
                    mask8x64::from_bitmask(self.bit_counts_total[bit_index][i])
                        .to_int()
                        .cast()
                        * u64x64::splat(mul as u64);
            }
            self.bit_counts_total[bit_index] = u64x4::splat(0);
        }
    }
    // returns the count of the added ships
    pub fn add_possible_ship_positions(&mut self, ship_positions: u64x4) -> u64 {
        unsafe {
            *self
                .bit_fields_to_sum
                .get_unchecked_mut(self.added_bit_fields) = ship_positions;
        }
        self.added_bit_fields += 1;

        let mut added_ships = 0;
        for i in 0..4 {
            // added_ships += 17;
            added_ships += ship_positions[i].count_ones() as u64;
            // dbg!(ship_positions[i].count_ones() as u64);
        }
        added_ships
    }
}

fn bit_adder_in_place<const N: usize, const M: usize>(vals: &mut [u64x4; N], b: &[u64x4; M]) {
    let mut carry = u64x4::splat(0);
    assert!(M <= N);
    for i in 0..M {
        let a = vals[i];
        vals[i] = a ^ b[i] ^ carry;
        carry = (a & b[i]) | (carry & (a ^ b[i]));
        // b[i] = u64x4::splat(0);
    }
    for i in M..N {
        let a = vals[i];
        vals[i] = a ^ carry;
        carry &= a
    }
}

fn bit_sum_test(val: &[u64x4; 15]) -> [u64x4; 4] {
    let a = bit_adder_with_carry([val[0]], [val[1]], val[2]);
    let b = bit_adder_with_carry([val[3]], [val[4]], val[5]);
    let e = bit_adder_with_carry(a, b, val[6]);
    let c = bit_adder_with_carry([val[8]], [val[9]], val[10]);
    let d = bit_adder_with_carry([val[11]], [val[12]], val[13]);
    let f = bit_adder_with_carry(c, d, val[14]);
    bit_adder_with_carry(e, f, val[7])
}
#[inline(always)]
fn bit_sum_test_2(val: &[u64x4; 63]) -> [u64x4; 6] {
    let a = bit_sum_test(val[0..15].try_into().unwrap());
    let b = bit_sum_test(val[16..16 + 15].try_into().unwrap());
    let e = bit_adder_with_carry(a, b, val[15]);
    let c = bit_sum_test(val[32..32 + 15].try_into().unwrap());
    let d = bit_sum_test(val[48..48 + 15].try_into().unwrap());
    let f = bit_adder_with_carry(c, d, val[15 + 32]);
    bit_adder_with_carry(e, f, val[31])
}
fn bit_sum_test_3(val: &[u64x4; 255]) -> [u64x4; 8] {
    let a = bit_sum_test_2(val[0..63].try_into().unwrap());
    let b = bit_sum_test_2(val[64..64 + 63].try_into().unwrap());
    let e = bit_adder_with_carry(a, b, val[63]);
    let c = bit_sum_test_2(val[128..128 + 63].try_into().unwrap());
    let d = bit_sum_test_2(val[192..192 + 63].try_into().unwrap());
    let f = bit_adder_with_carry(c, d, val[63 + 128]);
    bit_adder_with_carry(e, f, val[63 + 64])
}

fn sum_test(vals: &[u64x4; 255], sum_low: &mut [u64x4; 6]) {
    let mut sum_high = [u64x4::splat(0); 6];
    unroll! {
        for i in 0..255 {
            let mut carry = vals[i];

            let n = i + 1;
            let is_low_carry = n.count_ones() == n.trailing_ones();
            let bits_to_sum = n.trailing_ones() as usize - is_low_carry as usize;
            // dbg!(i);
            for sum_i in 0..bits_to_sum {
                // dbg!(sum_i);
                unsafe {
                    carry = single_add_with_carry(
                        sum_low.get_unchecked_mut(sum_i),
                        sum_high.get_unchecked_mut(sum_i),
                        carry,
                    )
                }
            }
            unsafe {
                if is_low_carry {
                    *sum_low.get_unchecked_mut(bits_to_sum) = carry;
                } else {
                    *sum_high.get_unchecked_mut(bits_to_sum) = carry;
                }
            }
        }
    }
}
fn single_add(a: &mut u64x4, carry: u64x4) -> u64x4 {
    let new_carry = *a & carry;
    *a ^= carry;
    new_carry
}
fn single_add_with_carry(a: &mut u64x4, b: &u64x4, carry: u64x4) -> u64x4 {
    let res = *a ^ b ^ carry;
    let new_carry = (*a & b) | (carry & (*a ^ b));
    *a = res;
    new_carry
}

fn bit_sum<const N: usize, const M: usize>(
    vals_to_sum: &[[u64x4; N]; M],
) -> [[u64x4; N + 1]; M / 2] {
    assert!(const { M.is_power_of_two() });
    let mut res = [[u64x4::splat(0); N + 1]; M / 2];
    for ([a, b], res) in zip(vals_to_sum.array_chunks::<2>(), &mut res) {
        *res = bit_adder(*a, *b)
    }
    res
}
fn bit_adder_with_carry<const N: usize>(
    a: [u64x4; N],
    b: [u64x4; N],
    mut carry: u64x4,
) -> [u64x4; N + 1] {
    let mut res = [u64x4::splat(0); N + 1];

    for i in 0..N {
        res[i] = a[i] ^ b[i] ^ carry;
        carry = (a[i] & b[i]) | (carry & (a[i] ^ b[i]))
    }
    res[N] = carry;
    res
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
pub struct BoardCounts<const N: usize> {
    pub counts: [u64; BOARD_SIZE],
    pub board_count: u64,
    pub ship_cell_counts: ShipCellCounts<N>,
}

impl<const N: usize> BoardCounts<N> {
    pub fn new() -> Self {
        BoardCounts {
            counts: [0; BOARD_SIZE],
            board_count: 0,
            ship_cell_counts: ShipCellCounts::new(),
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
    pub fn add(mut self, other: Self) -> Self {
        for (self_count, other_count) in zip(&mut self.counts, &other.counts) {
            *self_count += *other_count;
        }
        self.board_count += other.board_count;
        self
    }
    #[inline(never)]
    pub fn sum_cell_counts(&mut self, ship_counts: ShipCounts) {
        self.ship_cell_counts.sum_last_ship();
        for (ship, cell_counts) in zip(
            ship_counts.iter_ships(),
            &mut self.ship_cell_counts.counts_per_ship_position,
        ) {
            cell_counts.sum_bit_counts();
            let counts = cell_counts.counts();
            for i in 0..100 {
                let ship_count_x = counts[i];
                let ship_count_y = counts[i + 128];
                for ship_i in 0..ship.length() {
                    self.counts[i + ship_i] += ship_count_x;
                    if i + ship_i * 10 < 128 {
                        self.counts[i + ship_i * 10] += ship_count_y;
                    }
                }
            }
            *cell_counts = CellCount::new();
        }
    }
    pub fn print_colorfull(&self) {
        println!();

        let max_val = *self.counts.iter().max().unwrap() as f32;

        let color_grad = colorgrad::preset::rd_yl_gn();

        for row in self.counts.chunks(SIZE).take(SIZE) {
            for count in row {
                let probability = *count as f32 / (self.board_count as f32);

                let color_scale = *count as f32 / max_val;

                let rgba8 = color_grad.at(color_scale).to_rgba8();

                let colored_string = format_args!("{:3.0}", probability * 1000.)
                    .to_string()
                    .on_truecolor(rgba8[0], rgba8[1], rgba8[2])
                    .black();
                print!("{colored_string} ");
            }
            println!();
        }
    }
}

impl<const N: usize> Display for BoardCounts<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        for row in self.counts.chunks(SIZE).take(SIZE) {
            for count in row {
                let counts = *count as f32;
                let probability = counts / (self.board_count as f32);
                f.write_fmt(format_args!("{:3.1} ", probability * 100.))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
