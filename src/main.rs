#![feature(
    portable_simd,
    adt_const_params,
    stdarch_x86_avx512,
    slice_as_chunks,
    bigint_helper_methods
)]
#![allow(dead_code, clippy::new_without_default)]
// #![warn(clippy::pedantic)]

mod board;

use std::time::Duration;

use bit_board::BitBoard;
use board::Board;
use ship::Ship;
use ship_counts::ShipCountsSmall;
use solver::{PlacedBitShips, Solver, SpecialRng};

const SIZE: usize = 10;
const BOARD_SIZE: usize = (SIZE * SIZE).next_multiple_of(64);

const SHIPS: &[Ship] = &[
    Ship::new(4),
    Ship::new(3),
    Ship::new(3),
    Ship::new(2),
    Ship::new(2),
    Ship::new(2),
    Ship::new(1),
    Ship::new(1),
    Ship::new(1),
    Ship::new(1),
];

mod bit_board;
mod ship;
mod ship_counts;
mod solver;

fn main() {
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     .build_global()
    //     .unwrap();

    let mut solver = Solver::new();

    let time_to_run = Duration::from_millis(1000);
    loop {
        solver.run(time_to_run);
        solver.reset();
    }
}

#[inline(never)]
#[rustfmt::skip]
pub fn step(
    bit_board: BitBoard,
    ship_amounts: [u8; 4],
    ship_counts: &mut ship_counts::ShipCounts,
    placed_bit_ships: &PlacedBitShips,
    special_rng: &mut SpecialRng,
) {
    const LOOP_PARALLELISM: usize = 6;
    const LOOP_ITERATIONS: usize = 255 / LOOP_PARALLELISM;

    let iterations = LOOP_ITERATIONS * LOOP_PARALLELISM;

    let mut small_counts = ShipCountsSmall::new();

    // amortise the cost of the time comparison of the loop outside the function. Gives 10 % speedup
    for _ in 0..LOOP_ITERATIONS { // only can sum up to 255 in the ship_counts
        // random_place ship is short enough to fit allow multiple executions in the cpu at once without dependency on the previous random_place_ship
        let mut boards = [bit_board; LOOP_PARALLELISM];


        if ship_amounts[3] > 0 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(4) }>(placed_bit_ships, special_rng);  }  }
        if ship_amounts[2] > 0 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(3) }>(placed_bit_ships, special_rng);  }  }
        if ship_amounts[2] > 1 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(3) }>(placed_bit_ships, special_rng);  }  }
        if ship_amounts[1] > 0 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng);  }  }
        if ship_amounts[1] > 1 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng);  }  }
        if ship_amounts[1] > 2 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng);  }  }
        if ship_amounts[0] > 0 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng);  }  }
        if ship_amounts[0] > 1 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng);  }  }
        if ship_amounts[0] > 2 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng);  }  }
        if ship_amounts[0] > 3 {  for board in &mut boards {  board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng);  }  }


        // if ship_amounts[3] > 0 { board.random_place_ship::<{ Ship::new(4) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[2] > 0 { board.random_place_ship::<{ Ship::new(3) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[1] > 0 { board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[0] > 0 { board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng); }

        // if ship_amounts[3] > 0 { board.random_place_ship::<{ Ship::new(4) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[2] > 0 { board.random_place_ship::<{ Ship::new(3) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[2] > 1 { board.random_place_ship::<{ Ship::new(3) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[1] > 0 { board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[1] > 1 { board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[1] > 2 { board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[0] > 0 { board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[0] > 1 { board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[0] > 2 { board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng); }
        // if ship_amounts[0] > 3 { board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng); }

        for board in &boards {
            small_counts.add_bit_board(*board);
        }
        // ship_counts.add_bit_board(board);
    }
    ship_counts.add_small_counts(small_counts, iterations as u64);
}
