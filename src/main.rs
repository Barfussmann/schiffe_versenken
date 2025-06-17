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

#[allow(unused_imports)]
use std::sync::atomic::{AtomicU64, Ordering};

use board::Board;
use glam::IVec2;
#[allow(unused_imports)]
use num_format::{Locale, ToFormattedString};
#[allow(unused_imports)]
use rayon::iter::{IntoParallelIterator, ParallelDrainRange, ParallelIterator};

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

    let mut solver = DynSolver::new(ShipCounts::new([1, 1, 1, 1]), Board::new(), IVec2::ZERO);
    let depth = 70;
    let start_time = std::time::Instant::now();
    let total_arrangements = solver.calculate_arrangements_in_depth(depth);
    println!(
        "{depth:>2}: total_arrangements: {:>12}, in: {:>8.1?}",
        total_arrangements.to_formatted_string(&Locale::en),
        start_time.elapsed(),
    );

    let mut solvers = vec![DynSolver::new(
        ShipCounts::new([1, 1, 1, 1]),
        Board::new(),
        IVec2::ZERO,
    )];
    for iteration in 0..21 {
        let start_time = std::time::Instant::now();
        let total_arrangements = AtomicU64::new(0);
        solvers = solvers
            .par_drain(..)
            .flat_map(|mut solver| {
                if solver.ship_counts.total_ships() == 0 {
                    return Vec::new();
                }
                let own_arrangements = solver.calculate_arrangements();
                total_arrangements.fetch_add(own_arrangements, Ordering::Relaxed);
                if let Some(best_cell) = solver.get_best_cell() {
                    let mut sub_solvers = solver.shoot(best_cell);
                    let sub_arrangements = sub_solvers
                        .iter_mut()
                        .map(|solver| solver.clone().calculate_arrangements())
                        .sum::<u64>();

                    if sub_arrangements != own_arrangements {
                        println!(
                            "own: {}, sub: {}, dif: {}",
                            own_arrangements.to_formatted_string(&Locale::en),
                            sub_arrangements.to_formatted_string(&Locale::en),
                            own_arrangements
                                .abs_diff(sub_arrangements)
                                .to_formatted_string(&Locale::en)
                        );
                        println!("Own Board: {}", solver.board);
                        for sub_board in &sub_solvers {
                            println!("Sub Board {}", sub_board.board);
                        }
                        sub_solvers.iter_mut().for_each(|solver| {
                            solver.clone().calculate_arrangements();
                        });
                        panic!()
                    }

                    return sub_solvers;
                }
                Vec::new()
            })
            .collect::<Vec<_>>();
        println!(
            "{iteration:>2}: total_arrangements: {:>12}, in: {:>8.1?}",
            total_arrangements
                .load(std::sync::atomic::Ordering::Relaxed)
                .to_formatted_string(&Locale::en),
            start_time.elapsed(),
        );
    }
}
