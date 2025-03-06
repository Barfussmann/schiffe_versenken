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
    rayon::ThreadPoolBuilder::new()
        .num_threads(16)
        .build_global()
        .unwrap();

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
    for _ in 0..100 {
        // amortise the cost of the time comparison. Gives 10 % speedup
        let mut boards = [bit_board; 7];
        // let mut board = [bit_board; 8];


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


        // board.random_place_ship::<{ Ship::new(4) }>(placed_bit_ships, special_rng);
        // board.random_place_ship::<{ Ship::new(3) }>(placed_bit_ships, special_rng);
        // board.random_place_ship::<{ Ship::new(3) }>(placed_bit_ships, special_rng);
        // board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng);
        // board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng);
        // board.random_place_ship::<{ Ship::new(2) }>(placed_bit_ships, special_rng);
        // board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng);
        // board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng);
        // board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng);
        // board.random_place_ship::<{ Ship::new(1) }>(placed_bit_ships, special_rng);
        //
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
            ship_counts.add_bit_board(*board);
        }
        // ship_counts.add_bit_board(board);
    }
}
