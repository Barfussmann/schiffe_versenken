#![allow(dead_code)]
use std::fmt::{Display, Write};
use std::{iter::zip, time::Instant};
mod board;

use board::Board;
use board::Cell;
use rand::Rng;
use rand::thread_rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const SIZE: usize = 10;
const BOARD_SIZE: usize = (SIZE * SIZE).next_multiple_of(64);

const SHIPS: [Ship; 10] = [
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

#[derive(Debug, Clone)]
struct ShipCounts {
    counts: [usize; BOARD_SIZE],
    board_count: usize,
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
                Cell::Protected => {}
                Cell::Water => {}
            }
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
struct Ship {
    length: usize,
    index: usize,
}

impl Ship {
    const fn new(length: usize, index: usize) -> Ship {
        Ship { length, index }
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

fn main() {
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     .build_global()
    //     .unwrap();
    loop {
        let start_time = Instant::now();
        // let ship_count = 90_000;
        // let ship_count = 5_000_000;
        let ship_count = 105_000_000;

        // let iterations = ship_count / 2usize.pow(SHIPS.len() as u32);
        //
        // let ship_counts = (0..1)
        // let ship_counts = (0..iterations)
        let ship_counts = (0..ship_count)
            .into_par_iter()
            .fold(ShipCounts::new, |mut ship_counts, _| {
                // let board = Board::new();
                let rng = &mut fastrand::Rng::new();
                // let rng = &mut BadRNG::new();
                // let mut board = Board::new();
                // board.place_all_ships_random_recursive(&SHIPS, &mut ship_counts, rng);
                // board.place_all_ships_recursive(0, 0, SHIPS, &mut ship_counts);

                let mut board = Board::new();
                for ship in SHIPS {
                    board.random_place_ship(ship, rng);
                }
                ship_counts.add_board(board);
                ship_counts
            })
            .collect::<Vec<_>>()
            .into_iter()
            .reduce(|mut a, b| {
                a.add_other_count(b);
                a
            })
            .unwrap();
        // for _ in 0..1_000_000 {
        //     ship_counts.add_board(board);
        // }
        let elapsed_time = start_time.elapsed();
        println!("{ship_count} took: {:?}", elapsed_time);
        println!("ship_counts: {}", ship_counts)
    }
}
