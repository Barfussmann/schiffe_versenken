use std::time::Instant;

use crate::{
    SIZE,
    bit_board::{BitBoard, PlacedBitShips},
    bit_iter::BitIter,
    board::{Board, Cell},
    board_counts::BoardCounts,
    ship::ShipCounts,
};
use num_format::{Locale, ToFormattedString};

pub struct Solver<const N: usize> {
    pub placed_bit_ships: PlacedBitShips<N>,
    pub current_board: Board,
    bit_board: BitBoard<N>,
    pub ship_counts: ShipCounts,
    pub board_counts: BoardCounts<N>,
}

impl<const N: usize> Solver<N> {
    pub fn new(ship_counts: ShipCounts, board: Board) -> Self {
        Solver {
            placed_bit_ships: PlacedBitShips::new(ship_counts),
            current_board: board,
            bit_board: board.to_bitboard(ship_counts),
            ship_counts,
            board_counts: BoardCounts::new(),
        }
    }

    pub fn reset(&mut self) {
        self.current_board = Board::new();
    }
    pub fn run(&mut self) {
        // let start_time = Instant::now();
        self.board_counts.board_count += self.place_ship_recursive::<0>(self.bit_board);
        self.board_counts.sum_cell_counts(self.ship_counts);

        // let (x, y) = self.get_best_water_cell();

        // let elapsed_time = start_time.elapsed();
        // let boards_per_second =
        //     (self.board_counts.board_count as f64 / elapsed_time.as_secs_f64()) as u64;
        // println!(
        //     "in: {elapsed_time:7.3?}, possibilities: {:12}, calculated: {:12} hz",
        //     self.board_counts
        //         .board_count
        //         .to_formatted_string(&Locale::en),
        //     boards_per_second.to_formatted_string(&Locale::en)
        // );
        // // self.board_counts.print_colorfull();
        // println!(
        //     "Average placed ships: {}",
        //     self.board_counts.counts.iter().sum::<u64>() as f64
        //         / self.board_counts.board_count as f64
        // );

        // println!("Max (x, y): ({}, {})", (x as u8 + b'A') as char, y + 1);
    }

    fn get_best_water_cell(&self) -> (usize, usize) {
        let max_index = self
            .board_counts
            .counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .unwrap()
            .0;
        assert!(self.current_board.cells[max_index] == Cell::Water);
        let (x, y) = (max_index % SIZE, max_index / SIZE);
        (x, y)
    }

    #[inline(always)]
    fn place_ship_recursive<const INDEX: usize>(&mut self, board: BitBoard<N>) -> u64 {
        // directly add the the positions of all possible placements of the last ship
        if INDEX + 1 == N {
            return self
                .board_counts
                .ship_cell_counts
                .add_possible_ship_positions(board.allowable::<INDEX>());
            // return self.board_counts.ship_cell_counts.counts_per_ship_position[INDEX]
            //     .add_possible_ship_positions(board.allowable::<INDEX>());
        }

        let mut configurations = 0;
        for ship_pos in BitIter::new(board.allowable::<INDEX>()) {
            let board = board.place_ship::<INDEX>(ship_pos, &self.placed_bit_ships);

            let additional_configurations = match INDEX {
                0 => self.place_ship_recursive::<1>(board),
                1 => self.place_ship_recursive::<2>(board),
                2 => self.place_ship_recursive::<3>(board),
                3 => self.place_ship_recursive::<4>(board),
                4 => self.place_ship_recursive::<5>(board),
                _ => unreachable!(),
            };
            // addes the currently placed ship with the count of sub-configurations
            self.board_counts.ship_cell_counts.counts_per_ship_position[INDEX]
                .add_single_ship(ship_pos, additional_configurations);

            configurations += additional_configurations;
        }

        // flush the cell counts to prevent overflow
        if INDEX + 2 == N {
            self.board_counts.ship_cell_counts.sum_last_ship();
        }
        configurations
    }
}
