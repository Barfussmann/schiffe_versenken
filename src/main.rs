#![feature(
    portable_simd,
    adt_const_params,
    generic_const_exprs,
    stdarch_x86_avx512,
    iter_array_chunks,
    array_chunks
)]
#![allow(
    dead_code,
    clippy::new_without_default,
    incomplete_features,
    clippy::needless_range_loop
)]
// #![allow(dead_code, clippy::new_without_default, unused, incomplete_features)]
// #![warn(clippy::pedantic)]
mod board;

use ship::ShipCounts;
use solver::Solver;

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

    // Solver::<1>::new(ShipCounts::new([1, 0, 0, 0])).run();
    // Solver::<1>::new(ShipCounts::new([0, 1, 0, 0])).run();
    // Solver::<1>::new(ShipCounts::new([0, 0, 1, 0])).run();
    // Solver::<1>::new(ShipCounts::new([0, 0, 0, 1])).run();

    // Solver::<2>::new(ShipCounts::new([1, 1, 0, 0])).run();
    // Solver::<2>::new(ShipCounts::new([1, 0, 1, 0])).run();
    // Solver::<2>::new(ShipCounts::new([0, 1, 1, 0])).run();
    // Solver::<2>::new(ShipCounts::new([0, 0, 2, 0])).run();
    // Solver::<2>::new(ShipCounts::new([1, 0, 0, 1])).run();
    // Solver::<2>::new(ShipCounts::new([0, 1, 0, 1])).run();
    // Solver::<2>::new(ShipCounts::new([0, 0, 1, 1])).run();

    // Solver::<3>::new(ShipCounts::new([1, 1, 1, 0])).run();
    // Solver::<3>::new(ShipCounts::new([1, 0, 2, 0])).run();
    // Solver::<3>::new(ShipCounts::new([0, 1, 2, 0])).run();
    // Solver::<3>::new(ShipCounts::new([1, 1, 0, 1])).run();
    // Solver::<3>::new(ShipCounts::new([1, 0, 1, 1])).run();
    // Solver::<3>::new(ShipCounts::new([0, 1, 1, 1])).run();
    // Solver::<3>::new(ShipCounts::new([0, 0, 2, 1])).run();

    // Solver::<4>::new(ShipCounts::new([1, 1, 2, 0])).run();
    // Solver::<4>::new(ShipCounts::new([1, 1, 1, 1])).run();
    // Solver::<4>::new(ShipCounts::new([1, 0, 2, 1])).run();
    // Solver::<4>::new(ShipCounts::new([0, 1, 2, 1])).run();

    // Solver::<5>::new(ShipCounts::new([1, 1, 2, 1])).run();
    for _ in 0..10 {
        Solver::<5>::new(ShipCounts::new([1, 1, 2, 1])).run();
    }
}
