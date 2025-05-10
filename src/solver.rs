use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use crate::{
    SIZE,
    bit_board::BitBoard,
    bit_iter::BitIter,
    board::{Board, Cell, Direction},
    board_counts::{BoardCounts, ShipPositionCounts},
    ship::Ship,
};
use num_format::{Locale, ToFormattedString};
use rayon::prelude::*;

pub struct PlacedBitShips {
    pub placed_ships: [[BitBoard; 256]; 6],
}
impl PlacedBitShips {
    pub fn new() -> &'static Self {
        const SHIPS: [Ship; 6] = [
            Ship::new(1, 0),
            Ship::new(2, 0),
            Ship::new(3, 0),
            Ship::new(4, 0),
            Ship::new(5, 0),
            Ship::new(6, 0),
        ];

        static PLACED_BIT_SHIPS: LazyLock<PlacedBitShips> = LazyLock::new(|| {
            assert!(
                SHIPS.is_sorted_by_key(|ship| ship.index()),
                "SHIPS has to be sorted by ship.index()"
            );
            let placed_ships = SHIPS.map(|ship| {
                let mut placed_ships = [BitBoard::new(Board::new()); 256];
                for dir in [Direction::Horizontal, Direction::Vertical] {
                    for y in 0..SIZE {
                        for x in 0..SIZE {
                            let bit_board_index = dir as usize * 128 + (y * 10 + x);
                            let mut board = Board::new();
                            board.const_place_ship(x, y, dir, ship);

                            placed_ships[bit_board_index] = BitBoard::new(board);
                        }
                    }
                }
                placed_ships
            });
            PlacedBitShips { placed_ships }
        });
        &PLACED_BIT_SHIPS
    }
}

pub struct Solver {
    pub placed_bit_ships: &'static PlacedBitShips,
    pub current_board: Board,
}

impl Solver {
    pub fn new() -> Self {
        Solver {
            placed_bit_ships: PlacedBitShips::new(),
            current_board: Board::new(),
        }
    }

    pub fn reset(&mut self) {
        self.current_board = Board::new();
    }
    pub fn run(&self, time_to_run: Duration, ship_amounts: [u8; 5]) {
        let start_time = Instant::now();
        let ship_counts = self.inner_loop(time_to_run, ship_amounts);

        let (x, y) = self.get_best_water_cell(&ship_counts);

        let elapsed_time = start_time.elapsed();
        println!(
            "in: {:4.3?} calculated: {:12}",
            elapsed_time,
            ship_counts.board_count.to_formatted_string(&Locale::en)
        );
        println!("ship_counts: {ship_counts}");
        println!(
            "Average placed ships: {}",
            ship_counts.counts.iter().sum::<u64>() as f64 / ship_counts.board_count as f64
        );

        println!("Max (x, y): ({}, {})", (x as u8 + b'A') as char, y + 1);
    }

    fn get_best_water_cell(&self, ship_counts: &BoardCounts) -> (usize, usize) {
        let max_index = ship_counts
            .counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .unwrap()
            .0;
        assert!(self.current_board.cells[max_index] == Cell::Water);
        // let max_index = zip(
        //     ship_counts.counts.iter().enumerate(),
        //     &self.current_board.cells,
        // )
        // .filter(|(_, cell)| **cell == Cell::Water)
        // .max_by_key(|((_, count), _)| **count)
        // .unwrap()
        // .0
        // .0;
        let (x, y) = (max_index % SIZE, max_index / SIZE);
        (x, y)
    }
    pub fn inner_loop(&self, time_to_run: Duration, ship_counts: [u8; 5]) -> BoardCounts {
        let bit_board = BitBoard::new(self.current_board);

        let start_time = Instant::now();
        (0..rayon::current_num_threads())
            .par_bridge()
            .map(|_| {
                let mut board_counts = BoardCounts::new();

                let end_time = start_time + time_to_run;
                while Instant::now() < end_time {
                    step(
                        bit_board,
                        ship_counts,
                        &mut board_counts,
                        self.placed_bit_ships,
                    );
                    // step_summing::<{ Ship::new(5, 4) }>(
                    //     bit_board,
                    //     &mut board_counts,
                    //     self.placed_bit_ships,
                    // );
                }
                board_counts
            })
            .reduce(BoardCounts::new, |mut a, b| {
                a.add_other_count(b);
                a
            })
    }
}

const SHIP_COUNT: usize = 5;
#[rustfmt::skip]
#[inline(always)]
fn step_inner<
    const SHIP: Ship,
    const NEXT_SHIP_INDEX: [usize; SHIP_COUNT],
>(
    board: BitBoard,
    counts: &mut [ShipPositionCounts; 5],
    placed_bit_ships: &PlacedBitShips,
) -> u64 {
    // directly add the the positions of all possible placements of the last ship
    if const { remaining_ships(SHIP.index(), NEXT_SHIP_INDEX) } == 0 {
        return counts[SHIP.index].add_possible_ship_positions(board.allowable::<SHIP>())
    }

    let mut configurations = 0;
    // for ship_pos in BitIter::new(board.allowable_dyn(ship)) {
    for ship_pos in BitIter::new(board.allowable::<SHIP>()) {
        let mut board = board;
        board.place_ship::<SHIP>(ship_pos, placed_bit_ships);


        let additional_configurations = match NEXT_SHIP_INDEX[SHIP.index] {
            0 => step_inner::<{ Ship::new(2, 0) }, NEXT_SHIP_INDEX>(board,  counts, placed_bit_ships),
            1 => step_inner::<{ Ship::new(3, 1) }, NEXT_SHIP_INDEX>(board,  counts, placed_bit_ships),
            2 => step_inner::<{ Ship::new(3, 2) }, NEXT_SHIP_INDEX>(board,  counts, placed_bit_ships),
            3 => step_inner::<{ Ship::new(4, 3) }, NEXT_SHIP_INDEX>(board,  counts, placed_bit_ships),
            4 => step_inner::<{ Ship::new(5, 4) }, NEXT_SHIP_INDEX>(board,  counts, placed_bit_ships),
            _ => unreachable!()
            // _ => {0}
        };
        // addes the currently placed ship with the amount of differnt configurations
        counts[SHIP.index].add_single_ship(ship_pos, additional_configurations);

        configurations += additional_configurations;
    }
    // flush the small count with the u8 to the big u64 nums to prevent overflow
    if const { remaining_ships(SHIP.index(), NEXT_SHIP_INDEX) } == 1 {
        {
            counts[NEXT_SHIP_INDEX[SHIP.index]].sum_bit_counts(7..8);
        };
    }
    configurations
}
const fn next_ship_length(placed_ships: usize, ship_counts: [u8; 5]) -> usize {
    let mut running_sum = 0;
    let mut length = 1;
    while length <= 5 {
        running_sum += ship_counts[length - 1] as usize;
        if running_sum > placed_ships {
            return length;
        }
        length += 1;
    }
    unreachable!()
}
const fn remaining_ships(ship_index: usize, next_ship_index: [usize; SHIP_COUNT]) -> usize {
    let mut ship_count = 0;
    let mut ship_index = ship_index;
    while ship_index < SHIP_COUNT {
        ship_index = next_ship_index[ship_index];
        ship_count += 1;
    }
    ship_count - 1
}

pub fn step(
    bit_board: BitBoard,
    ship_counts: [u8; 5],
    board_counts: &mut BoardCounts,
    placed_bit_ships: &PlacedBitShips,
) {
    match ship_counts {
        [_, 1, 2, 1, 1] => step_summing::<{ Ship::new(5, 4) }, { [255, 0, 1, 2, 3] }>(
            bit_board,
            board_counts,
            placed_bit_ships,
        ),

        rem => println!("Not implemented: {rem:?}"),
    }
}
#[inline(never)]
pub fn step_summing<const STARTING_SHIP: Ship, const NEXT_SHIP_INDEX: [usize; 5]>(
    bit_board: BitBoard,
    board_counts: &mut BoardCounts,
    placed_bit_ships: &PlacedBitShips,
) {
    let mut counts: [_; 5] = std::array::from_fn(|_| ShipPositionCounts::new());
    let total_boards =
        step_inner::<STARTING_SHIP, NEXT_SHIP_INDEX>(bit_board, &mut counts, placed_bit_ships);
    board_counts.board_count += total_boards;
    board_counts.add_ship_positions_counts::<{ Ship::new(2, 0) }>(&mut counts[0]);
    board_counts.add_ship_positions_counts::<{ Ship::new(3, 0) }>(&mut counts[1]);
    board_counts.add_ship_positions_counts::<{ Ship::new(3, 0) }>(&mut counts[2]);
    board_counts.add_ship_positions_counts::<{ Ship::new(4, 0) }>(&mut counts[3]);
    board_counts.add_ship_positions_counts::<{ Ship::new(5, 0) }>(&mut counts[4]);
}
