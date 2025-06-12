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
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     .build_global()
    //     .unwrap();

    // let mut terminal = ratatui::init();
    // let mut solver_render = solver_render::SolverRender::new(DynSolver::new(
    //     ShipCounts::new([1, 1, 2, 1]),
    //     &Board::new(),
    // ));
    // solver_render.run(&mut terminal);
    // ratatui::restore();

    // let mut board = Board::new();
    // board.cells[66] = Cell::Protected;
    // let mut solver = DynSolver::new(ShipCounts::new([1, 1, 2, 1]), board);
    // solver.step();
    let mut solver = DynSolver::new(ShipCounts::new([1, 1, 2, 1]), Board::new());
    solver.split_and_count();

    // board.

    // for board in Solver::<1>::new(ShipCounts::new([1, 0, 0, 0]), board).run() {
    // for (board, ship) in Solver::<1>::new(ShipCounts::new([1, 0, 0, 0]), board).run() {
    //     // for board in Solver::<5>::new(ShipCounts::new([1, 1, 2, 1]), board).run() {
    //     println!("{board}");
    // }
    // Solver::<1>::new(ShipCounts::new([1, 0, 0, 0]), board)
    //     .run()
    //     .count();

    // for _ in 0..100 {
    //     DynSolver::new(ShipCounts::new([1, 1, 2, 1]), &Board::new()).run();
    // }
}

// Solver::<1>::new(ShipCounts::new([0, 1, 0, 0]), Board::new()).run();
// Solver::<1>::new(ShipCounts::new([0, 0, 1, 0]), Board::new()).run();
// Solver::<1>::new(ShipCounts::new([0, 0, 0, 1]), Board::new()).run();

// Solver::<2>::new(ShipCounts::new([1, 1, 0, 0]), Board::new()).run();
// Solver::<2>::new(ShipCounts::new([1, 0, 1, 0]), Board::new()).run();
// Solver::<2>::new(ShipCounts::new([0, 1, 1, 0]), Board::new()).run();
// Solver::<2>::new(ShipCounts::new([0, 0, 2, 0]), Board::new()).run();
// Solver::<2>::new(ShipCounts::new([1, 0, 0, 1]), Board::new()).run();
// Solver::<2>::new(ShipCounts::new([0, 1, 0, 1]), Board::new()).run();
// Solver::<2>::new(ShipCounts::new([0, 0, 1, 1]), Board::new()).run();

// Solver::<3>::new(ShipCounts::new([1, 1, 1, 0]), Board::new()).run();
// Solver::<3>::new(ShipCounts::new([1, 0, 2, 0]), Board::new()).run();
// Solver::<3>::new(ShipCounts::new([0, 1, 2, 0]), Board::new()).run();
// Solver::<3>::new(ShipCounts::new([1, 1, 0, 1]), Board::new()).run();
// Solver::<3>::new(ShipCounts::new([1, 0, 1, 1]), Board::new()).run();
// Solver::<3>::new(ShipCounts::new([0, 1, 1, 1]), Board::new()).run();
// Solver::<3>::new(ShipCounts::new([0, 0, 2, 1]), Board::new()).run();

// Solver::<4>::new(ShipCounts::new([1, 1, 2, 0]), Board::new()).run();
// Solver::<4>::new(ShipCounts::new([1, 1, 1, 1]), Board::new()).run();
// Solver::<4>::new(ShipCounts::new([1, 0, 2, 1]), Board::new()).run();
// Solver::<4>::new(ShipCounts::new([0, 1, 2, 1]), Board::new()).run();

// Solver::<5>::new(ShipCounts::new([1, 1, 2, 1]), Board::new()).run();
