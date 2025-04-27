#![feature(
    portable_simd,
    adt_const_params,
    stdarch_x86_avx512,
    slice_as_chunks,
    bigint_helper_methods,
    stmt_expr_attributes
)]
// #![allow(clippy::new_without_default)]
#![allow(dead_code, clippy::new_without_default, unused)]
// #![warn(clippy::pedantic)]

mod board;
use core::{hint::unreachable_unchecked, iter::Iterator, simd::prelude::*};
use std::time::Duration;

#[allow(unused)]
use bit_board::{BitBoard, DoubleBitBoard, OctaBitBoard};
// use board::Board;
use ship::Ship;
use ship_counts::{ShipCountsSmall, ShipPositionCounts, ShipPositionCountsSmall};
use solver::{PlacedBitShips, Solver, SpecialRng};

const SIZE: usize = 10;
const BOARD_SIZE: usize = (SIZE * SIZE).next_multiple_of(64);

mod bit_board;
mod ship;
mod ship_counts;
mod solver;

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build_global()
        .unwrap();

    let mut solver = Solver::new();
    let ship_amounts = std::hint::black_box([0, 0, 0, 0, 1, 0]);
    // let ship_amounts = std::hint::black_box([0, 1, 2, 1, 1, 0]);
    // let ship_amounts = std::hint::black_box([4, 3, 2, 1, 0, 0]); // russian fleet

    let time_to_run = Duration::from_millis(1000);
    loop {
        solver.run(time_to_run, ship_amounts);
        // println!("{}", solver.current_board);
        solver.reset();
    }
}

const SHIP_COUNT: usize = 5;
#[rustfmt::skip]
fn sub_step<
    const SHIP_INDEX: usize,
    const SHIP: Ship,
    const NEXT_SHIP_INDEX: [usize; SHIP_COUNT],
>(
    board: BitBoard,
    ship_positions_counts: &mut [ShipPositionCounts; 5],
    ship_positions_counts_small: &mut ShipPositionCountsSmall,
    placed_bit_ships: &PlacedBitShips,
) -> u64 {
    // directly add the the positions of all possible placements of the last ship
    if const { remaining_ships::<SHIP_INDEX, NEXT_SHIP_INDEX>() } == 0 {
        return ship_positions_counts_small.add_possible_ship_positions(board.allowable::<SHIP>())
    }

    let mut configurations = 0;
    for ship_pos in BitIter::new(board.allowable::<SHIP>()) {
        let mut board = board;
        board.place_ship::<SHIP>(ship_pos, placed_bit_ships);

        let additional_configurations = match NEXT_SHIP_INDEX[SHIP_INDEX] {
            0 => sub_step::<0, { Ship::new(2) }, NEXT_SHIP_INDEX>(board, ship_positions_counts, ship_positions_counts_small, placed_bit_ships,),
            1 => sub_step::<1, { Ship::new(3) }, NEXT_SHIP_INDEX>(board, ship_positions_counts, ship_positions_counts_small, placed_bit_ships,),
            2 => sub_step::<2, { Ship::new(3) }, NEXT_SHIP_INDEX>(board, ship_positions_counts, ship_positions_counts_small, placed_bit_ships,),
            3 => sub_step::<3, { Ship::new(4) }, NEXT_SHIP_INDEX>(board, ship_positions_counts, ship_positions_counts_small, placed_bit_ships,),
            4 => sub_step::<4, { Ship::new(5) }, NEXT_SHIP_INDEX>(board, ship_positions_counts, ship_positions_counts_small, placed_bit_ships,),
            _ => {unsafe {unreachable_unchecked()}}
            // _ => {0}
        };
        // addes the currently placed ship with the amount of differnt configurations
        ship_positions_counts[SHIP_INDEX].add_single_ship(ship_pos, additional_configurations);

        configurations += additional_configurations;
    }
    // flush the small count with the u8 to the big u64 nums to prevent overflow
    if const { remaining_ships::<SHIP_INDEX, NEXT_SHIP_INDEX>() } == 1 {
        ship_positions_counts[NEXT_SHIP_INDEX[SHIP_INDEX]].add_small(ship_positions_counts_small);
    }
    configurations
}
const fn remaining_ships<const SHIP_INDEX: usize, const NEXT_SHIP_INDEX: [usize; SHIP_COUNT]>()
-> usize {
    let mut ship_count = 0;
    let mut ship_index = SHIP_INDEX;
    while ship_index < SHIP_COUNT {
        ship_index = NEXT_SHIP_INDEX[ship_index];
        ship_count += 1;
    }
    ship_count - 1
}

#[inline(never)]
pub fn step_iter(
    bit_board: BitBoard,
    ship_counts: &mut ship_counts::ShipCounts,
    placed_bit_ships: &PlacedBitShips,
) {
    let mut ship_positions_counts: [_; 5] = std::array::from_fn(|_| ShipPositionCounts::new());
    let total_boards = sub_step::<4, { Ship::new(5) }, { [255, 0, 1, 2, 3] }>(
        bit_board,
        &mut ship_positions_counts,
        &mut ShipPositionCountsSmall::new(),
        placed_bit_ships,
    );
    ship_counts.board_count += total_boards;
    ship_counts.add_ship_positions_counts::<{ Ship::new(2) }>(&mut ship_positions_counts[0]);
    ship_counts.add_ship_positions_counts::<{ Ship::new(3) }>(&mut ship_positions_counts[1]);
    ship_counts.add_ship_positions_counts::<{ Ship::new(3) }>(&mut ship_positions_counts[2]);
    ship_counts.add_ship_positions_counts::<{ Ship::new(4) }>(&mut ship_positions_counts[3]);
    ship_counts.add_ship_positions_counts::<{ Ship::new(5) }>(&mut ship_positions_counts[4]);
}

#[derive(Clone, Copy)]
struct BitIter {
    current_u64: u64,
    bits: u64x4,
    offset: u8,
    //     indecies: [u8; 256],
    //     len: u8,
}
impl BitIter {
    // #[inline(never)]
    fn new(bits: u64x4) -> Self {
        // let mut indecies = [0; 256];
        // let nums_0 = u8x64::from_array(std::array::from_fn(|i| i as u8));
        // let nums_1 = u8x64::from_array(std::array::from_fn(|i| i as u8 + 64));
        // let nums_2 = u8x64::from_array(std::array::from_fn(|i| i as u8 + 128));
        // let nums_3 = u8x64::from_array(std::array::from_fn(|i| i as u8 + 192));

        // let offset_1 = bits[0].count_ones();
        // let offset_2 = bits[0].count_ones() + bits[1].count_ones();
        // let offset_3 = bits[0].count_ones() + bits[1].count_ones() + bits[2].count_ones();
        // let len = (bits[0].count_ones()
        //     + bits[1].count_ones()
        //     + bits[2].count_ones()
        //     + bits[3].count_ones()) as u8;

        // #[rustfmt::skip]
        // unsafe {
        //     _mm512_mask_compressstoreu_epi8(indecies.as_mut_ptr(), bits[0], nums_0.into());
        //     _mm512_mask_compressstoreu_epi8(indecies.as_mut_ptr().add(offset_1 as usize), bits[1], nums_1.into());
        //     _mm512_mask_compressstoreu_epi8(indecies.as_mut_ptr().add(offset_2 as usize), bits[2], nums_2.into());
        //     _mm512_mask_compressstoreu_epi8(indecies.as_mut_ptr().add(offset_3 as usize), bits[3], nums_3.into());
        // }

        Self {
            current_u64: bits[0],
            bits,
            offset: 0,
            // indecies,
            // len,
        }
    }
    fn dummy() -> Self {
        Self::new(u64x4::splat(0))
    }
}
impl Iterator for BitIter {
    type Item = u8;

    // #[inline(never)]
    fn next(&mut self) -> core::prelude::v1::Option<Self::Item> {
        // let ret = Some(self.indecies[self.offset as usize]);
        // self.offset += 1;
        // if self.offset > self.len {
        //     return None;
        // }
        // ret

        loop {
            let bit_index = self.current_u64.trailing_zeros() as u8;
            if bit_index == 64 {
                if self.offset == 64 * 3 {
                    return None;
                }
                self.offset += 64;
                self.current_u64 = self.bits[self.offset as usize / 64];
                continue;
            }
            self.current_u64 ^= 1 << bit_index;
            return Some(self.offset + bit_index);
        }
    }
}

#[inline(never)]
#[rustfmt::skip]
pub fn step(
    bit_board: BitBoard,
    ship_amounts: [u8; 6],
    ship_counts: &mut ship_counts::ShipCounts,
    placed_bit_ships: &PlacedBitShips,
    special_rng: &mut SpecialRng,
) {
    // const LOOP_PARALLELISM: usize = 1;
    // const INSTRUCTION_PARALLELISM: usize = OctaBitBoard::INSTRUCTION_PARALLELISM;
    // let octa_bit_board = OctaBitBoard::new(bit_board);

    const LOOP_PARALLELISM: usize = 4;
    const INSTRUCTION_PARALLELISM: usize = DoubleBitBoard::INSTRUCTION_PARALLELISM;
    let double_bit_board = DoubleBitBoard::new(bit_board);

    // const LOOP_PARALLELISM: usize = 1;
    // const INSTRUCTION_PARALLELISM: usize = BitBoard::INSTRUCTION_PARALLELISM;

    const LOOP_ITERATIONS: usize = 255 / LOOP_PARALLELISM / INSTRUCTION_PARALLELISM;

    let mut small_counts = ShipCountsSmall::new();

    // amortise the cost of the time comparison of the loop outside the function. Gives 10 % speedup
    for _ in 0..LOOP_ITERATIONS { // only can sum up to 255 in the ship_counts
        // random_place ship is short enough to fit allow multiple executions in the cpu at once without dependency on the previous random_place_ship
        // let mut boards = [octa_bit_board; LOOP_PARALLELISM];
        let mut boards = [double_bit_board; LOOP_PARALLELISM];
        // let mut boards = [bit_board; LOOP_PARALLELISM];


        for _ in 0..ship_amounts[5] {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(6) }>(placed_bit_ships, special_rng);  }  }
        for _ in 0..ship_amounts[4] {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(5) }>(placed_bit_ships, special_rng);  }  }
        for _ in 0..ship_amounts[3] {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(4) }>(placed_bit_ships, special_rng);  }  }
        for _ in 0..ship_amounts[2] {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(3) }>(placed_bit_ships, special_rng);  }  }
        for _ in 0..ship_amounts[1] {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng);  }  }
        for _ in 0..ship_amounts[0] {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng);  }  }

        for board in &boards {
            // small_counts.add_octa_bit_board(*board);
            small_counts.add_double_bit_board(*board);
            // small_counts.add_bit_board(*board);
        }
        // ship_counts.add_bit_board(board);
    }
    ship_counts.add_small_counts(&mut small_counts);
}
