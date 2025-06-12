#![feature(
    portable_simd,
    adt_const_params,
    int_roundings,
    generic_const_exprs,
    stdarch_x86_avx512,
    iter_array_chunks,
    array_chunks,
    vec_push_within_capacity,
    iter_from_coroutine,
    yield_expr,
    coroutines
)]
#![allow(
    dead_code,
    clippy::new_without_default,
    incomplete_features,
    clippy::needless_range_loop
)]
// #![allow(dead_code, clippy::new_without_default, unused, incomplete_features)]
// #![warn(clippy::pedantic)]

use board::Board;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{ship::ShipCounts, solver::DynSolver};

const SIZE: usize = 10;
const BOARD_SIZE: usize = (SIZE * SIZE).next_multiple_of(64);

mod bit_board;
mod bit_iter;
mod board;
mod board_counts;
mod cell;
mod ship;
mod solver;
mod solver_render;
mod utils;

fn main() {
    // let mut terminal = ratatui::init();
    // let mut solver_render = solver_render::SolverRender::new(DynSolver::new(
    //     ShipCounts::new([1, 1, 2, 1]),
    //     Board::new(),
    // ));
    // solver_render.run(&mut terminal);
    // ratatui::restore();

    // let mut solver = DynSolver::new(ShipCounts::new([1, 1, 2, 1]), Board::new());
    // solver.split_and_count();

    for _ in 0..100 {
        DynSolver::new(ShipCounts::new([1, 1, 2, 1]), Board::new())
            .calculate_board_counts_with_prints();
    }
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(16)
    //     .build_global()
    //     .unwrap();
    // (0..10000).into_par_iter().for_each(|_| {
    //     DynSolver::new(ShipCounts::new([1, 1, 2, 1]), Board::new())
    //         .calculate_board_counts_with_prints();
    // });
}
