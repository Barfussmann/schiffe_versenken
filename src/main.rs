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
#[allow(unused_imports)]
use num_format::{Locale, ToFormattedString};
#[allow(unused_imports)]
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
    // {
    //     let mut terminal = ratatui::init();
    //     let mut solver_render = solver_render::SolverRender::new(DynSolver::new(
    //         ShipCounts::new([1, 1, 2, 1]),
    //         Board::new(),
    //     ));
    //     solver_render.run(&mut terminal);
    //     ratatui::restore();
    // }

    let mut solvers = vec![DynSolver::new(ShipCounts::new([1, 1, 2, 1]), Board::new())];
    let mut new_solvers = Vec::new();
    for _ in 0..7 {
        let mut total_arrangements = 0;
        for mut solver in solvers.drain(..) {
            println!("{}", solver.current_board);
            total_arrangements += solver.calculate_arrangements();
            println!("{}", solver.cell_hit_count_sum);
            new_solvers.extend(solver.shoot(solver.get_best_cell()));
        }
        println!(
            "total_arrangements: {:>14}",
            total_arrangements.to_formatted_string(&Locale::en)
        );
        std::mem::swap(&mut solvers, &mut new_solvers);
    }
}
