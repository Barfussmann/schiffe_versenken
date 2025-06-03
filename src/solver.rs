use std::{
    simd::{num::SimdUint, u64x4},
    time::Instant,
};

use crate::{
    SIZE,
    bit_board::{BitBoard, PlacedBitShips},
    bit_iter::BitIter,
    board::{Board, Cell},
    board_counts::BoardCounts,
    ship::ShipCounts,
};
use loop_code::repeat;
use num_format::{Locale, ToFormattedString};

pub struct Solver<const N: usize> {
    pub placed_bit_ships: PlacedBitShips<N>,
    pub current_board: Board,
    bit_board: BitBoard<N>,
    pub ship_counts: ShipCounts,
    board_counts: BoardCounts<N>,
    final_ships_to_place: [Vec<([u8; N], u64x4, u64x4)>; 128],
    // final_ships_to_place: [Vec<([u8; N], u64x4)>; 128],
}

impl<const N: usize> Solver<N> {
    pub fn new(ship_counts: ShipCounts, board: Board) -> Self {
        Solver {
            placed_bit_ships: PlacedBitShips::new(ship_counts),
            current_board: board,
            bit_board: board.to_bitboard(ship_counts),
            ship_counts,
            board_counts: BoardCounts::new(),
            final_ships_to_place: std::array::from_fn(|_| Vec::with_capacity(1024)),
        }
    }

    pub fn reset(&mut self) {
        self.current_board = Board::new();
    }
    pub fn run(&mut self)
    where
        [(); N - 1]:,
        [(); N - 2]:,
    {
        let start_time = Instant::now();
        self.place_ship_recursive::<0>(self.bit_board, [0; N]);
        self.board_counts.sum_cell_counts(self.ship_counts);

        let (x, y) = self.get_best_water_cell();

        let elapsed_time = start_time.elapsed();
        let boards_per_second =
            (self.board_counts.board_count as f64 / elapsed_time.as_secs_f64()) as u64;
        println!(
            "in: {elapsed_time:7.3?}, possibilities: {:12}, calculated: {:12} hz",
            self.board_counts
                .board_count
                .to_formatted_string(&Locale::en),
            boards_per_second.to_formatted_string(&Locale::en)
        );
        self.board_counts.print_colorfull();
        println!(
            "Average placed ships: {}",
            self.board_counts.counts.iter().sum::<u64>() as f64
                / self.board_counts.board_count as f64
        );

        println!("Max (x, y): ({}, {})", (x as u8 + b'A') as char, y + 1);
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
    fn place_ship_recursive<const INDEX: usize>(
        &mut self,
        board: BitBoard<N>,
        mut ship_indecies: [u8; N],
    ) where
        [(); INDEX + 1]:,
        [(); N - 1]:,
        [(); N - 2]:,
    {
        if INDEX + 2 == N {
            let allowable = board.allowable::<INDEX>();
            let count = allowable.count_ones().reduce_sum();

            // unsafe {
            //     self.final_ships_to_place
            //         .get_unchecked_mut(count as usize)
            //         .push_within_capacity((ship_indecies, board))
            //         .unwrap_unchecked()
            // }
            self.final_ships_to_place[count as usize].push((
                ship_indecies,
                allowable,
                *board.protected.last().unwrap(),
            ));
            return;
        }

        for ship_pos in BitIter::new(board.allowable::<INDEX>()) {
            let board = board.place_ship::<INDEX>(ship_pos, &self.placed_bit_ships);
            ship_indecies[INDEX] = ship_pos;

            match INDEX {
                0 => self.place_ship_recursive::<1>(board, ship_indecies),
                1 => self.place_ship_recursive::<2>(board, ship_indecies),
                2 => self.place_ship_recursive::<3>(board, ship_indecies),
                3 => self.place_ship_recursive::<4>(board, ship_indecies),
                4 => self.place_ship_recursive::<5>(board, ship_indecies),
                _ => unreachable!(),
            };
        }

        if INDEX == 1 {
            self.sum_cached_bitfields();
        }
    }

    #[inline(never)]
    fn sum_cached_bitfields(&mut self)
    where
        [(); N - 1]:,
        [(); N - 2]:,
    {
        self.board_counts
            .ship_cell_counts
            .counts_per_ship_position
            .last_mut()
            .unwrap()
            .bit_fields_to_sum
            .fill(u64x4::splat(0));
        repeat!(INDEX 100 {
            for (ship_poses, allowable, last_allowable) in self.final_ships_to_place[INDEX].drain(..) {
                let mut configurations = 0;
                for ship_pos in BitIter::new(allowable) {
                    let last_allowable = last_allowable & *unsafe {
                        self.placed_bit_ships
                        .placed_ships
                        .last()
                        .unwrap_unchecked()
                        .get_unchecked(ship_pos as usize)
                        .protected
                        .last()
                        .unwrap_unchecked()
                    };
                    let additional_configurations = self.board_counts.ship_cell_counts.counts_per_ship_position[N - 1]
                        .add_possible_ship_positions(last_allowable);

                    self.board_counts.ship_cell_counts.counts_per_ship_position[N - 2]
                        .add_single_ship(ship_pos, additional_configurations);
                    configurations += additional_configurations;
                }
                for (i, ship_pos) in ship_poses.into_iter().enumerate().take(N - 2) {
                    self.board_counts.ship_cell_counts.counts_per_ship_position[i]
                        .add_single_ship(ship_pos, configurations);
                }
                self.board_counts.ship_cell_counts.sum_last_ship::<INDEX>();

                self.board_counts.board_count += configurations;
            }
        });

        // for boards in &mut self.final_ships_to_place {
        //     for (ship_poses, board) in boards.drain(..) {
        //         let mut configurations = 0;
        //         for ship_pos in BitIter::new(board.allowable::<{ N - 2 }>()) {
        //             let board = board.place_ship::<{ N - 2 }>(ship_pos, &self.placed_bit_ships);
        //             let allowable = board.allowable::<{ N - 1 }>();
        //             let additional_configurations = allowable.count_ones().reduce_sum();

        //             self.board_counts.ship_cell_counts.counts_per_ship_position[N - 2]
        //                 .add_single_ship(ship_pos, additional_configurations);
        //             configurations += additional_configurations;
        //         }
        //         for (i, ship_pos) in ship_poses.into_iter().enumerate().take(N - 2) {
        //             self.board_counts.ship_cell_counts.counts_per_ship_position[i]
        //                 .add_single_ship(ship_pos, configurations);
        //         }
        //         self.board_counts.board_count += configurations;
        //     }
        // }
    }
}
