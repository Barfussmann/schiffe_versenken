use colored::Colorize;
use colorgrad::Gradient;
use loop_code::repeat;

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
    pub fn sum_last_ship<const COUNT: usize>(&mut self) {
        self.counts_per_ship_position
            .last_mut()
            .unwrap()
            .sum_single_bits::<COUNT>();
    }
}

#[derive(Debug, Clone)]
pub struct CellCount {
    position_counts: [u64x64; 4],
    bit_counts: [u64x4; 8],
    pub bit_counts_total: [u64x4; 32],
    added_bit_fields: usize,
    pub bit_fields_to_sum: [u64x4; 512],
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
    // #[inline(never)]
    pub fn sum_single_bits<const COUNT: usize>(&mut self) {
        let bit_fields_to_sum: &[u64x4; 127] =
            self.bit_fields_to_sum[0..127].first_chunk::<127>().unwrap();

        let sum = bit_sum_fn_6::<COUNT>(bit_fields_to_sum, 0);
        // let sum = bit_sum_6(bit_fields_to_sum);
        bit_adder_in_place(&mut self.bit_counts_total, &sum);

        self.added_bit_fields = 0;
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

        ship_positions.count_ones().reduce_sum()
    }
}

pub fn bit_adder_in_place<const N: usize, const M: usize>(vals: &mut [u64x4; N], b: &[u64x4; M]) {
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

#[inline(always)]
#[rustfmt::skip]
fn bit_sum_fn_1<const N: usize>(vals: &[u64x4; 127], index: usize) -> [u64x4; 2] {
    let mut new_vals = [u64x4::splat(0); 3];
    if index     < N { new_vals[0] = vals[index    ] }
    if index + 1 < N { new_vals[1] = vals[index + 1] }
    if index + 2 < N { new_vals[2] = vals[index + 2] }

    bit_adder_with_carry([new_vals[0]], [new_vals[1]], new_vals[2])
}
#[inline(always)]
fn bit_sum_fn_2<const N: usize>(vals: &[u64x4; 127], index: usize) -> [u64x4; 3] {
    let a = bit_sum_fn_1::<N>(vals, index);
    let b = bit_sum_fn_1::<N>(vals, index + 3);
    let carry = if index + 6 < N {
        vals[index + 6]
    } else {
        u64x4::splat(0)
    };
    bit_adder_with_carry(a, b, carry)
}
#[inline(always)]
fn bit_sum_fn_3<const N: usize>(vals: &[u64x4; 127], index: usize) -> [u64x4; 4] {
    let a = bit_sum_fn_2::<N>(vals, index);
    let b = bit_sum_fn_2::<N>(vals, index + 7);
    let carry = if index + 14 < N {
        vals[index + 14]
    } else {
        u64x4::splat(0)
    };

    bit_adder_with_carry(a, b, carry)
}
#[inline(always)]
fn bit_sum_fn_4<const N: usize>(vals: &[u64x4; 127], index: usize) -> [u64x4; 5] {
    let a = bit_sum_fn_3::<N>(vals, index);
    let b = bit_sum_fn_3::<N>(vals, index + 15);
    let carry = if index + 30 < N {
        vals[index + 30]
    } else {
        u64x4::splat(0)
    };

    bit_adder_with_carry(a, b, carry)
}
#[inline(always)]
fn bit_sum_fn_5<const N: usize>(vals: &[u64x4; 127], index: usize) -> [u64x4; 6] {
    let a = bit_sum_fn_4::<N>(vals, index);
    let b = bit_sum_fn_4::<N>(vals, index + 31);
    let carry = if index + 62 < N {
        vals[index + 62]
    } else {
        u64x4::splat(0)
    };
    bit_adder_with_carry(a, b, carry)
}
#[inline(never)]
fn bit_sum_fn_6<const N: usize>(vals: &[u64x4; 127], index: usize) -> [u64x4; 7] {
    let a = bit_sum_fn_5::<N>(vals, index);
    let b = bit_sum_fn_5::<N>(vals, index + 63);
    let carry = if index + 126 < N {
        vals[index + 126]
    } else {
        u64x4::splat(0)
    };
    bit_adder_with_carry(a, b, carry)
}
#[inline(always)]
fn bit_sum_fn_7<const N: usize>(vals: &[u64x4; 127], index: usize) -> [u64x4; 8] {
    let a = bit_sum_fn_6::<N>(vals, index);
    let b = bit_sum_fn_6::<N>(vals, index + 127);
    let carry = if index + 254 < N {
        vals[index + 254]
    } else {
        u64x4::splat(0)
    };
    bit_adder_with_carry(a, b, carry)
}
#[inline(never)]
pub fn bit_sum_fn_dyn(count: usize, vals: &[u64x4; 127]) -> [u64x4; 7] {
    repeat!(INDEX 100 {
        if count == INDEX {
            return bit_sum_fn_6::<INDEX>(vals, 0);
            // return bit_sum_fn_7::<INDEX>(0, fun);
        }
    });
    unreachable!()
}

#[inline(always)]
fn bit_sum_1(vals: &[u64x4; 3]) -> [u64x4; 2] {
    bit_adder_with_carry([vals[0]], [vals[1]], vals[2])
}

#[inline(always)]
fn bit_sum_2(vals: &[u64x4; 7]) -> [u64x4; 3] {
    let a = bit_sum_1(vals[0..3].try_into().unwrap());
    let b = bit_sum_1(vals[3..6].try_into().unwrap());
    bit_adder_with_carry(a, b, vals[6])
}
#[inline(always)]
fn bit_sum_3(vals: &[u64x4; 15]) -> [u64x4; 4] {
    let a = bit_sum_2(vals[0..7].try_into().unwrap());
    let b = bit_sum_2(vals[7..14].try_into().unwrap());
    bit_adder_with_carry(a, b, vals[14])
}
#[inline(always)]
fn bit_sum_4(vals: &[u64x4; 31]) -> [u64x4; 5] {
    let a = bit_sum_3(vals[0..15].try_into().unwrap());
    let b = bit_sum_3(vals[15..30].try_into().unwrap());
    bit_adder_with_carry(a, b, vals[30])
}
#[inline(always)]
fn bit_sum_5(vals: &[u64x4; 63]) -> [u64x4; 6] {
    let a = bit_sum_4(vals[0..31].try_into().unwrap());
    let b = bit_sum_4(vals[31..62].try_into().unwrap());
    bit_adder_with_carry(a, b, vals[62])
}
#[inline(always)]
fn bit_sum_6(vals: &[u64x4; 127]) -> [u64x4; 7] {
    let a = bit_sum_5(vals[0..63].try_into().unwrap());
    let b = bit_sum_5(vals[63..126].try_into().unwrap());
    bit_adder_with_carry(a, b, vals[126])
}
#[inline(always)]
fn bit_sum_7(vals: &[u64x4; 255]) -> [u64x4; 8] {
    let a = bit_sum_6(vals[0..127].try_into().unwrap());
    let b = bit_sum_6(vals[127..254].try_into().unwrap());
    bit_adder_with_carry(a, b, vals[254])
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
