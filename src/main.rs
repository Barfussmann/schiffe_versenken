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

    const SHOULD_PLACE_SHIP: [bool; 5] = [true; 5];

    const F: bool = false;
    const T: bool = true;

    Solver::<{ [T, F, F, F, F] }>::new().run();
    Solver::<{ [F, T, F, F, F] }>::new().run();
    Solver::<{ [F, F, F, T, F] }>::new().run();
    Solver::<{ [F, F, F, F, T] }>::new().run();
    Solver::<{ [T, T, F, F, F] }>::new().run();
    Solver::<{ [T, F, F, T, F] }>::new().run();
    Solver::<{ [F, T, F, T, F] }>::new().run();
    Solver::<{ [F, F, T, T, F] }>::new().run();
    Solver::<{ [T, F, F, F, T] }>::new().run();
    Solver::<{ [F, T, F, F, T] }>::new().run();
    Solver::<{ [F, F, F, T, T] }>::new().run();
    Solver::<{ [T, T, F, T, F] }>::new().run();
    Solver::<{ [T, F, T, T, F] }>::new().run();
    Solver::<{ [F, T, T, T, F] }>::new().run();
    Solver::<{ [T, T, F, F, T] }>::new().run();
    Solver::<{ [T, F, F, T, T] }>::new().run();
    Solver::<{ [F, T, F, T, T] }>::new().run();
    Solver::<{ [F, F, T, T, T] }>::new().run();
    Solver::<{ [T, T, T, T, F] }>::new().run();
    Solver::<{ [T, T, F, T, T] }>::new().run();
    Solver::<{ [T, F, T, T, T] }>::new().run();
    Solver::<{ [F, T, T, T, T] }>::new().run();
    Solver::<{ [T, T, T, T, T] }>::new().run();
}
