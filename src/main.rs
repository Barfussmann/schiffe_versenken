#![feature(
    portable_simd,
    adt_const_params,
    generic_const_exprs,
    stdarch_x86_avx512
)]
#![allow(dead_code, clippy::new_without_default, unused, incomplete_features)]
// #![warn(clippy::pedantic)]

mod board;
use core::{hint::unreachable_unchecked, iter::Iterator, simd::prelude::*, unreachable};
use std::time::Duration;

#[allow(unused)]
use bit_board::BitBoard;
// use board::Board;
use board_counts::{BoardShipPositionCounts, ShipCountsSmall, ShipPositionCounts};
use ship::Ship;
use solver::{PlacedBitShips, Solver, SpecialRng};

const SIZE: usize = 10;
const BOARD_SIZE: usize = (SIZE * SIZE).next_multiple_of(64);

mod bit_board;
mod bit_iter;
mod board_counts;
mod ship;
mod solver;

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build_global()
        .unwrap();

    let mut solver = Solver::new();
    // let ship_amounts = std::hint::black_box([0, 0, 0, 0, 1]);
    let ship_amounts = std::hint::black_box([0, 1, 2, 1, 1]);
    // let ship_amounts = std::hint::black_box([4, 3, 2, 1, 0]); // russian fleet

    let time_to_run = Duration::from_millis(1000);
    loop {
        solver.run(time_to_run, ship_amounts);
        // println!("{}", solver.current_board);
        solver.reset();
    }
}
