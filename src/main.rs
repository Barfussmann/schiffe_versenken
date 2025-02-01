#![feature(portable_simd)]
#![allow(dead_code)]
use std::fmt::{Display, Write};
use std::{iter::zip, time::Instant};
mod board;

use board::Cell;
use board::{BitBoard, Board, set_protecet_at_offsets};
use num_format::{Locale, ToFormattedString};
use rand::Rng;
use rand::thread_rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::slice::ParallelSlice;

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
//     // Ship::new(4, 0),
//     Ship::new(4, 1),
//     // Ship::new(1, 2),
//     // Ship::new(1, 3),
//     // Ship::new(1, 4),
//     // Ship::new(1, 5),
//     // Ship::new(1, 6),
//     // Ship::new(1, 7),
//     // Ship::new(1, 8),
//     // Ship::new(1, 9),
// ];

#[derive(Debug, Clone)]
struct ShipCounts {
    counts: [u64; BOARD_SIZE],
    board_count: u64,
}

impl ShipCounts {
    fn new() -> ShipCounts {
        ShipCounts {
            counts: [0; BOARD_SIZE],
            board_count: 0,
        }
    }
    fn add_board(&mut self, board: Board) {
        for (count, cell) in zip(&mut self.counts, &board.cells) {
            match cell {
                Cell::Ship => {
                    *count += 1;
                }
                Cell::Protected | Cell::Water | Cell::ShipHit => {}
            }
        }
        self.board_count += 1;
    }
    #[inline(never)]
    fn add_bit_board(&mut self, board: BitBoard) {
        for i in 0..40 {
            self.counts[i] += ((board.ship[0] & (1 << i)) != 0) as u64;
        }
        // high bits
        for i in 0..60 {
            self.counts[i + 40] += ((board.ship[1] & (1 << i)) != 0) as u64;
        }

        self.board_count += 1;
    }
    fn add_other_count(&mut self, other: Self) {
        for (self_count, other_count) in zip(&mut self.counts, &other.counts) {
            *self_count += *other_count;
        }
        self.board_count += other.board_count;
    }
}
impl Display for ShipCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        for row in self.counts.chunks(SIZE).take(SIZE) {
            for count in row {
                let probability = (*count as f64) / (self.board_count as f64);

                f.write_fmt(format_args!("{:4.3} ", probability))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShipLength {
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ship {
    length: ShipLength,
    index: usize,
}

impl Ship {
    const fn new(length: usize, index: usize) -> Ship {
        let length = match length {
            1 => ShipLength::_1,
            2 => ShipLength::_2,
            3 => ShipLength::_3,
            4 => ShipLength::_4,
            5 => unreachable!(), // implement y shift by 5 in allowable_ship_placements. Currently it doesn't work because it would have to get a second amount data from high
            // 5 => ShipLength::_5,
            _ => unreachable!(),
        };
        Ship { length, index }
    }
    const fn length(&self) -> usize {
        self.length as usize
    }
}
pub struct BadRNG(u64);
impl BadRNG {
    fn new() -> Self {
        let val: u64 = thread_rng().r#gen();
        Self(val)
    }
    #[inline]
    pub fn gen_u64(&mut self) -> u64 {
        // Constants for WyRand taken from: https://github.com/wangyi-fudan/wyhash/blob/master/wyhash.h#L151
        // Updated for the final v4.2 implementation with improved constants for better entropy output.
        const WY_CONST_0: u64 = 0x2d35_8dcc_aa6c_78a5;
        const WY_CONST_1: u64 = 0x8bb8_4b93_962e_acc9;

        let s = self.0.wrapping_add(WY_CONST_0);
        self.0 = s;
        let t = u128::from(s) * u128::from(s ^ WY_CONST_1);
        (t as u64) ^ (t >> 64) as u64
    }
}

const DIAGONAL_OFFSETS: [(i32, i32); 4] = [(-1, -1), (1, -1), (-1, 1), (1, 1)];
const ORTHOGONAL_OFFSETS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

#[allow(unused_mut)]
fn main() {
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     .build_global()
    //     .unwrap();

    let iterations = 25_000_000u64;
    let random_values: Box<[u32]> = (0..iterations * SHIPS.len() as u64)
        .into_par_iter()
        .map(|_| rand::thread_rng().r#gen())
        .collect();

    let mut start_board = Board::new();
    let mut start_ships = SHIPS.to_vec();
    loop {
        let start_time = Instant::now();
        // let iterations = 10_000_000u64;

        let bit_board = BitBoard::new(start_board);

        let mut board = bit_board;
        let rng = &mut fastrand::Rng::new();
        for ship in &start_ships {
            board.random_place_ship(*ship, rng.u32(..), rng);
        }

        let ship_counts = random_values[..]
            .par_chunks(start_ships.len())
            // let ship_counts = (0..iterations)
            //     .into_par_iter()
            .fold(
                || (ShipCounts::new(), fastrand::Rng::new()),
                |(mut ship_counts, mut rng), rand_values| {
                    step(
                        &start_ships,
                        bit_board,
                        &mut ship_counts,
                        rand_values,
                        &mut rng,
                    );
                    #[inline(never)]
                    fn step(
                        start_ships: &[Ship],
                        bit_board: BitBoard,
                        ship_counts: &mut ShipCounts,
                        rand_values: &[u32],
                        rng: &mut fastrand::Rng,
                    ) {
                        let mut board = bit_board;

                        for (ship, random_value) in zip(start_ships, rand_values) {
                            board.random_place_ship(*ship, *random_value, rng);
                        }

                        ship_counts.add_bit_board(board);
                    }
                    (ship_counts, rng)
                },
            )
            .map(|(ship_counts, _)| ship_counts)
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
