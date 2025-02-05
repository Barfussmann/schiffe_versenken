#![feature(portable_simd, adt_const_params, stdarch_x86_avx512)]
#![allow(dead_code)]

use std::{iter::zip, time::Instant};
mod board;

use bit_board::BitBoard;
use board::Board;
use board::Cell;
use num_format::{Locale, ToFormattedString};
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::slice::ParallelSlice;
use ship::Ship;

const SIZE: usize = 10;
const BOARD_SIZE: usize = (SIZE * SIZE).next_multiple_of(64);

const SHIPS: &[Ship] = &[
    Ship::new(4, 0),
    Ship::new(3, 1),
    Ship::new(3, 2),
    Ship::new(2, 3),
    Ship::new(2, 4),
    Ship::new(2, 5),
    Ship::new(1, 6),
    Ship::new(1, 7),
    Ship::new(1, 8),
    Ship::new(1, 9),
];
// const SHIPS: &[Ship] = &[
//     Ship::new(4, 0),
//     Ship::new(3, 1),
//     Ship::new(3, 2),
//     Ship::new(2, 3),
//     Ship::new(2, 4),
//     Ship::new(2, 5),
//     Ship::new(1, 6),
//     Ship::new(1, 7),
//     Ship::new(1, 8),
//     Ship::new(1, 9),
// ];

mod bit_board;
mod ship;
mod ship_counts;

#[allow(unused_mut)]
fn main() {
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     .build_global()
    //     .unwrap();

    let iterations = 20_000_000u64;
    // let iterations = 140_000_000u64;

    let mut start_board = Board::new();
    let mut start_ships = SHIPS.to_vec();
    loop {
        let random_values: Box<[u32]> = (0..iterations * SHIPS.len() as u64)
            .into_par_iter()
            .map(|_| rand::thread_rng().r#gen())
            .collect();

        let bit_board = BitBoard::new(start_board);

        let start_time = Instant::now();

        let ship_counts = random_values[..]
            .par_chunks_exact(start_ships.len())
            // let ship_counts = (0..iterations)
            //     .into_par_iter()
            .fold(
                ship_counts::ShipCounts::new,
                |mut ship_counts, rand_values| {
                    step(
                        &start_ships,
                        bit_board,
                        &mut ship_counts,
                        rand_values.try_into().unwrap(),
                    );
                    #[inline(never)]
                    fn step(
                        _start_ships: &[Ship],
                        bit_board: BitBoard,
                        ship_counts: &mut ship_counts::ShipCounts,
                        random_values: &[u32; 10],
                    ) {
                        let mut board = bit_board;

                        board.random_place_ship::<{ Ship::new(4, 0) }>(random_values[0]);
                        // board.random_place_ship::<{ Ship::new(3, 1) }>(random_values[1]);
                        // board.random_place_ship::<{ Ship::new(3, 2) }>(random_values[2]);
                        // board.random_place_ship::<{ Ship::new(2, 3) }>(random_values[3]);
                        // board.random_place_ship::<{ Ship::new(2, 4) }>(random_values[4]);
                        // board.random_place_ship::<{ Ship::new(2, 5) }>(random_values[5]);
                        // board.random_place_ship::<{ Ship::new(1, 6) }>(random_values[6]);
                        // board.random_place_ship::<{ Ship::new(1, 7) }>(random_values[7]);
                        // board.random_place_ship::<{ Ship::new(1, 8) }>(random_values[8]);
                        // board.random_place_ship::<{ Ship::new(1, 9) }>(random_values[9]);

                        // for (ship, random_value) in zip(start_ships, rand_values) {
                        //     board.random_place_ship(*ship, *random_value);
                        // }

                        ship_counts.add_bit_board(board);
                    }
                    ship_counts
                },
            )
            .collect::<Vec<_>>()
            .into_iter()
            .reduce(|mut a, b| {
                a.add_other_count(b);
                a
            })
            .unwrap();

        let max_index = zip(ship_counts.counts.iter().enumerate(), &start_board.cells)
            .filter(|(_, cell)| **cell == Cell::Water)
            .max_by_key(|((_, count), _)| **count)
            .unwrap()
            .0
            .0;

        let elapsed_time = start_time.elapsed();
        println!(
            "{} took: {:?}",
            iterations.to_formatted_string(&Locale::de),
            elapsed_time
        );
        println!("ship_counts: {}", ship_counts);
        println!(
            "total_ships: {}",
            ship_counts.counts.iter().sum::<u64>() as f64 / ship_counts.board_count as f64
        );

        let x = max_index % SIZE;
        let y = max_index / SIZE;

        println!("Max (x, y): ({}, {})", (x as u8 + b'A') as char, y + 1);

        // println!("Hit(h), Kill(k), Miss(m):");
        // let mut answer = String::new();
        // std::io::stdin().read_line(&mut answer).unwrap();
        // let mut kill = false;
        // let hit_cell_type = match answer.trim() {
        //     "h" => Cell::ShipHit,
        //     "m" => Cell::Protected,
        //     "k" => {
        //         kill = true;
        //         Cell::Ship
        //     }
        //     _ => {
        //         println!("Invalid input. Assuming no hit.");
        //         Cell::Protected
        //     }
        // };
        // if hit_cell_type == Cell::Ship || hit_cell_type == Cell::ShipHit {
        //     set_protecet_at_offsets(x, y, &mut start_board, DIAGONAL_OFFSETS);
        // }
        // if kill {
        //     let mut hit_pos = vec![(x, y)];

        //     'outer: loop {
        //         set_protecet_at_offsets(x, y, &mut start_board, ORTHOGONAL_OFFSETS);
        //         start_board.cells[Board::cell_index(x, y)] = Cell::Ship;
        //         for orthogonal_offset in ORTHOGONAL_OFFSETS {
        //             let new_x = (x as i32 + orthogonal_offset.0) as usize;
        //             let new_y = (y as i32 + orthogonal_offset.1) as usize;
        //             if !(0..SIZE).contains(&new_x)
        //                 | !(0..SIZE).contains(&new_y)
        //                 | hit_pos.contains(&(new_x, new_y))
        //             {
        //                 continue;
        //             }
        //             if start_board.cells[Board::cell_index(new_x, new_y)] == Cell::ShipHit {
        //                 hit_pos.push((new_x, new_y));
        //                 continue 'outer;
        //             }
        //         }
        //         break;
        //     }
        //     let ship_len = hit_pos.len();
        //     println!("ship to remove: {ship_len}");
        //     for i in 0..start_ships.len() {
        //         if start_ships[i].length() == ship_len {
        //             start_ships.remove(i);
        //             break;
        //         }
        //     }
        //     println!("remaining_ships: {:?}", start_ships);
        // }
        // start_board.cells[max_index] = hit_cell_type;
        // println!("Board: {}", start_board);
    }
}

const DIAGONAL_OFFSETS: [(i32, i32); 4] = [(-1, -1), (1, -1), (-1, 1), (1, 1)];
const ORTHOGONAL_OFFSETS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
