#![feature(portable_simd, adt_const_params, stdarch_x86_avx512)]
#![allow(dead_code)]
// #![warn(clippy::pedantic)]

use std::io::Write;
use std::sync::LazyLock;
use std::{iter::zip, time::Instant};
mod board;

use bit_board::BitBoard;
use board::Cell;
use board::{Board, PLACED_SHIPS};
use rand::Rng;
#[cfg(not(feature = "rayon"))]
use rand::{SeedableRng, rngs::SmallRng};
#[cfg(feature = "rayon")]
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};
use ship::Ship;

const SIZE: usize = 10;
const BOARD_SIZE: usize = (SIZE * SIZE).next_multiple_of(64);

const SHIPS: &[Ship] = &[
    Ship::new(4, 0),
    Ship::new(3, 1),
    Ship::new(3, 1),
    Ship::new(2, 2),
    Ship::new(2, 2),
    Ship::new(2, 2),
    Ship::new(1, 3),
    Ship::new(1, 3),
    Ship::new(1, 3),
    Ship::new(1, 3),
];

mod bit_board;
mod ship;
mod ship_counts;

#[inline(never)]
fn step(
    _start_ships: &[Ship],
    bit_board: BitBoard,
    ship_counts: &mut ship_counts::ShipCounts,
    random_values: &[u32; 10],
) {
    let mut board = bit_board;

    board.random_place_ship::<{ Ship::new(4, 0) }>(random_values[0]);
    board.random_place_ship::<{ Ship::new(3, 1) }>(random_values[1]);
    board.random_place_ship::<{ Ship::new(3, 1) }>(random_values[2]);
    board.random_place_ship::<{ Ship::new(2, 2) }>(random_values[3]);
    board.random_place_ship::<{ Ship::new(2, 2) }>(random_values[4]);
    board.random_place_ship::<{ Ship::new(2, 2) }>(random_values[5]);
    board.random_place_ship::<{ Ship::new(1, 3) }>(random_values[6]);
    board.random_place_ship::<{ Ship::new(1, 3) }>(random_values[7]);
    board.random_place_ship::<{ Ship::new(1, 3) }>(random_values[8]);
    board.random_place_ship::<{ Ship::new(1, 3) }>(random_values[9]);

    // println!("{}", board);
    // for (ship, random_value) in zip(start_ships, rand_values) {
    //     board.random_place_ship(*ship, *random_value);
    // }

    ship_counts.add_bit_board(board);
}
#[inline(never)]
fn oh_no_this_will_panic() {
    panic!("Oh no a panic")
}
#[allow(arithmetic_overflow)]
#[inline(never)]
fn oh_no_a_overflow() -> u8 {
    129u8 + 130u8
    // 129u8 + 129u8
}

#[allow(unused_mut)]
fn main() {
    #[cfg(feature = "panic")]
    oh_no_this_will_panic();
    #[cfg(feature = "debug_assert")]
    debug_assert!(false, "oh no a debug assert went wrong");
    #[cfg(feature = "overflow")]
    oh_no_a_overflow();
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     .build_global()
    //     .unwrap();

    // let placed_bits_ships: LazyLock<Box<[[BitBoard; 256]; 4]>> = LazyLock::new(|| {
    //     let mut placed_ships = [[BitBoard::new(Board::new()); 256]; 4];
    //     for ship_i in 0..PLACED_SHIPS.len() {
    //         for dir in 0..2 {
    //             for i in 0..128 {
    //                 let bit_board_offset = BitBoard::map_index_to_bit_index(i);
    //                 let bit_board_index = dir * 128 + bit_board_offset;

    //                 let index = dir * 128 + i;
    //                 if bit_board_index < 256 {
    //                     placed_ships[ship_i][bit_board_index] =
    //                         BitBoard::new(PLACED_SHIPS[ship_i][index]);
    //                 }
    //             }
    //         }
    //     }
    //     Box::new(placed_ships)
    // });

    let iterations = 24_000_000u64;
    // let iterations = 140_000_000u64;

    let mut start_board = Board::new();
    let mut start_ships = SHIPS.to_vec();

    // #[cfg(not(feature = "rayon"))]
    // let random_values: Box<[u32]> = (0..iterations * SHIPS.len() as u64)
    //     .map(|_| rand::thread_rng().r#gen())
    //     .collect();
    #[cfg(not(feature = "rayon"))]
    let small_rng = &mut SmallRng::seed_from_u64(213485723049);
    #[cfg(not(feature = "rayon"))]
    let random_values: Box<[u32]> = (0..iterations * SHIPS.len() as u64)
        .map(|_| small_rng.r#gen())
        .collect();
    #[cfg(feature = "rayon")]
    let random_values: Box<[u32]> = (0..iterations * SHIPS.len() as u64)
        .into_par_iter()
        .map(|_| rand::thread_rng().r#gen())
        .collect();

    loop {
        let bit_board = BitBoard::new(start_board);

        let start_time = Instant::now();

        let ship_counts = inner_loop(&start_ships, &random_values, bit_board);

        let max_index = zip(ship_counts.counts.iter().enumerate(), &start_board.cells)
            .filter(|(_, cell)| **cell == Cell::Water)
            .max_by_key(|((_, count), _)| **count)
            .unwrap()
            .0
            .0;

        #[cfg(feature = "print_formatting")]
        {
            let elapsed_time = start_time.elapsed();
            println!("took: {:4.2?}", elapsed_time);
            println!("ship_counts: {}", ship_counts);
            println!(
                "total_ships: {}",
                ship_counts.counts.iter().sum::<u64>() as f64 / ship_counts.board_count as f64
            );
        }

        let x = max_index % SIZE;
        let y = max_index / SIZE;

        let mut buf = *b"Max (x, y): (A,  0)";
        buf[13] = x as u8 + b'A';
        if y == 10 {
            buf[16] = b'1';
        } else {
            buf[17] = y as u8 + b'1';
        }
        std::io::stdout().lock().write_all(&buf).unwrap();

        // println!("Max (x, y): ({}, {})", (x as u8 + b'A') as char, y + 1);
    }
}

fn inner_loop(
    start_ships: &[Ship],
    random_values: &[u32],
    bit_board: BitBoard,
) -> ship_counts::ShipCounts {
    #[cfg(not(feature = "rayon"))]
    {
        random_values[..].chunks(start_ships.len()).fold(
            ship_counts::ShipCounts::new(),
            |mut ship_counts, rand_values| {
                step(
                    start_ships,
                    bit_board,
                    &mut ship_counts,
                    rand_values.try_into().unwrap(),
                );

                ship_counts
            },
        )
    }
    #[cfg(feature = "rayon")]
    {
        random_values[..]
            .par_chunks_exact(start_ships.len())
            .fold(
                ship_counts::ShipCounts::new,
                |mut ship_counts, rand_values| {
                    step(
                        start_ships,
                        bit_board,
                        &mut ship_counts,
                        rand_values.try_into().unwrap(),
                    );

                    ship_counts
                },
            )
            .collect::<Vec<_>>()
            .into_iter()
            .reduce(|mut a, b| {
                a.add_other_count(b);
                a
            })
            .unwrap()
    }
}
