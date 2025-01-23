#![feature(adt_const_params)]
#![allow(dead_code)]
use std::{iter::zip, time::Instant};
mod board;
mod cell_grid;
mod ship_counts;

use board::Board;
use board::Cell;
use rand::Rng;
use rand::thread_rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use ship_counts::ShipCounts;

const SIZE: usize = 10;
const BOARD_SIZE: usize = SIZE.next_power_of_two().pow(2);

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

const DIAGONAL_OFFSETS: [(i32, i32); 4] = [(-1, -1), (1, -1), (-1, 1), (1, 1)];
const ORTHOGONAL_OFFSETS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

fn main() {
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     .build_global()
    //     .unwrap();

    let mut start_board = Board::new(Cell::Water);
    let mut start_ships = SHIPS.to_vec();
    let target_ship_cell_count: usize = start_ships.iter().map(|ship| ship.length).sum();
    loop {
        let start_time = Instant::now();
        // let ship_count = 50_000;
        // let ship_count = 1;
        let ship_count = 10_000_000;

        println!("target_ship_count: {target_ship_cell_count}");
        let ship_counts = (0..ship_count)
            .into_par_iter()
            .fold(ShipCounts::new, |mut ship_counts, _| {
                let rng = &mut fastrand::Rng::new();
                let mut board = start_board;

                // let mut board = Board::new();
                for ship in &start_ships {
                    // println!("{board}");
                    board.random_place_ship(*ship, rng);
                }

                let set_ship_cell_count = board
                    .cells
                    .as_flattened()
                    .iter()
                    .filter(|cell| **cell == Cell::Ship)
                    .count();

                if set_ship_cell_count == target_ship_cell_count {
                    ship_counts.add_board(board);
                }
                ship_counts
            })
            .collect::<Vec<_>>()
            .into_iter()
            .reduce(|mut a, b| {
                a.add_other_count(b);
                a
            })
            .unwrap();

        let _any_ship_hit = start_board
            .cells
            .as_flattened()
            .iter()
            .any(|cell| *cell == Cell::ShipHit);

        let max_index = zip(
            ship_counts.counts.iter().enumerate(),
            start_board.cells.as_flattened(),
        )
        .filter(|(_, cell)| **cell == Cell::Water)
        .max_by_key(|((_, count), _)| **count)
        .unwrap()
        .0
        .0;

        let elapsed_time = start_time.elapsed();
        println!("{ship_count} took: {:?}", elapsed_time);
        println!("ship_counts: {}", ship_counts);

        let x = max_index % 16;
        let y = max_index / 16;

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
            start_board.foreach_diagonal_neighbors(x, y, |cell| cell.protect());
            // start_board.set_protecet_at_offsets(x, y, &mut start_board, DIAGONAL_OFFSETS);
        }
        if kill {
            let mut hit_pos = vec![(x, y)];

            'outer: loop {
                start_board.foreach_orthogonal_neighbors(x, y, |cell| cell.protect());
                start_board[(x, y)] = Cell::Ship;

                for cords in Board::iter_offset_cords::<ORTHOGONAL_OFFSETS>(x, y) {
                    if hit_pos.contains(&cords) {
                        continue;
                    }
                    if start_board[cords] == Cell::ShipHit {
                        hit_pos.push(cords);
                        continue 'outer;
                    }
                }

                // start_board.orthogonal_neighbors(x, y).

                // for orthogonal_offset in ORTHOGONAL_OFFSETS {
                //     let new_x = (x as i32 + orthogonal_offset.0) as usize;
                //     let new_y = (y as i32 + orthogonal_offset.1) as usize;
                //     if !(0..SIZE).contains(&new_x)
                //         | !(0..SIZE).contains(&new_y)
                //         | hit_pos.contains(&(new_x, new_y))
                //     {
                //         continue;
                //     }
                //     if start_board[(new_x, new_y)] == Cell::ShipHit {
                //         hit_pos.push((new_x, new_y));
                //         continue 'outer;
                //     }
                // }
                break;
            }
            let ship_len = hit_pos.len();
            for i in 0..start_ships.len() {
                if start_ships[i].length == ship_len {
                    start_ships.remove(i);
                    break;
                }
            }
        }
        start_board[(x, y)] = hit_cell_type;
        println!("Board: {}", start_board);
    }
}
