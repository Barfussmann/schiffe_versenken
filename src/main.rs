#![allow(dead_code)]
use std::fmt::{Display, Write};
use std::{iter::zip, time::Instant};
mod board;

use board::Cell;
use board::{BitBoard, Board, set_protecet_at_offsets};
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
    fn add_bit_board(&mut self, board: BitBoard) {
        for i in 0..u128::BITS as usize {
            self.counts[i] += ((board.ship & (1 << i)) != 0) as u64;
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
            5 => ShipLength::_5,
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

fn main() {
    // println!("{:?}", nth_set_bit as *const u8);
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     .build_global()
    //     .unwrap();

    let mut start_board = Board::new();
    let mut start_ships = SHIPS.to_vec();
    let target_ship_cell_count: usize = start_ships.iter().map(|ship| ship.length()).sum();
    loop {
        let start_time = Instant::now();
        // let ship_count = 50_000;
        let ship_count = 10_000_000u64;
        // let ship_count = 25_000_000u64;
        // let ship_count = 25_000_000u64;

        let bit_board = BitBoard::new(start_board);
        println!("target_ship_count: {target_ship_cell_count}");
        let ship_counts = (0..ship_count)
            .into_par_iter()
            .fold(
                || (ShipCounts::new(), fastrand::Rng::new()),
                |(mut ship_counts, mut rng), _| {
                    step(&start_ships, bit_board, &mut ship_counts, &mut rng);
                    #[inline(never)]
                    fn step(
                        start_ships: &Vec<Ship>,
                        bit_board: BitBoard,
                        ship_counts: &mut ShipCounts,
                        rng: &mut fastrand::Rng,
                    ) {
                        let mut board = bit_board;

                        for ship in start_ships {
                            board.random_place_ship(*ship, rng);
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
        println!("{ship_count} took: {:?}", elapsed_time);
        println!("ship_counts: {}", ship_counts);

        let x = max_index % SIZE;
        let y = max_index / SIZE;

        println!("Max (x, y): ({}, {})", (x as u8 + b'A') as char, y + 1);

        println!("Hit(h), Kill(k), Miss(m):");
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer).unwrap();
        let mut kill = false;
        let hit_cell_type = match answer.trim() {
            "h" => Cell::ShipHit,
            "m" => Cell::Protected,
            "k" => {
                kill = true;
                Cell::Ship
            }
            _ => {
                println!("Invalid input. Assuming no hit.");
                Cell::Protected
            }
        };
        if hit_cell_type == Cell::Ship || hit_cell_type == Cell::ShipHit {
            set_protecet_at_offsets(x, y, &mut start_board, DIAGONAL_OFFSETS);
        }
        if kill {
            let mut hit_pos = vec![(x, y)];

            'outer: loop {
                set_protecet_at_offsets(x, y, &mut start_board, ORTHOGONAL_OFFSETS);
                start_board.cells[Board::cell_index(x, y)] = Cell::Ship;
                for orthogonal_offset in ORTHOGONAL_OFFSETS {
                    let new_x = (x as i32 + orthogonal_offset.0) as usize;
                    let new_y = (y as i32 + orthogonal_offset.1) as usize;
                    if !(0..SIZE).contains(&new_x)
                        | !(0..SIZE).contains(&new_y)
                        | hit_pos.contains(&(new_x, new_y))
                    {
                        continue;
                    }
                    if start_board.cells[Board::cell_index(new_x, new_y)] == Cell::ShipHit {
                        hit_pos.push((new_x, new_y));
                        continue 'outer;
                    }
                }
                break;
            }
            let ship_len = hit_pos.len();
            println!("ship to remove: {ship_len}");
            for i in 0..start_ships.len() {
                if start_ships[i].length() == ship_len {
                    start_ships.remove(i);
                    break;
                }
            }
            println!("remaining_ships: {:?}", start_ships);
        }
        start_board.cells[max_index] = hit_cell_type;
        println!("Board: {}", start_board);
    }
}
