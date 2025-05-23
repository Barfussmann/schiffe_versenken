use std::time::Instant;

use crate::{
    SIZE,
    bit_board::{BitBoard, PlacedBitShips, count_ships_to_place, ship_count_to_ship_index},
    bit_iter::BitIter,
    board::{Board, Cell},
    board_counts::{BoardCounts, CellCounts},
    ship::Ship,
};
use num_format::{Locale, ToFormattedString};

pub const SHIP_COUNT: usize = 5;
pub const SHIPS: [Ship; 5] = [
    Ship::new(5),
    Ship::new(4),
    Ship::new(3),
    Ship::new(3),
    Ship::new(2),
];

pub struct Solver<const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT]>
where
    [(); count_ships_to_place(SHOULD_PLACE_SHIP)]:,
{
    pub placed_bit_ships: PlacedBitShips<SHOULD_PLACE_SHIP>,
    pub current_board: Board,
}

impl<const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT]> Solver<SHOULD_PLACE_SHIP>
where
    [(); count_ships_to_place(SHOULD_PLACE_SHIP)]:,
{
    pub fn new() -> Self {
        Solver {
            placed_bit_ships: PlacedBitShips::new(),
            current_board: Board::new(),
        }
    }

    pub fn reset(&mut self) {
        self.current_board = Board::new();
    }
    pub fn run(&self) {
        let start_time = Instant::now();
        let board_counts = self.inner_loop();

        let (x, y) = self.get_best_water_cell(&board_counts);

        let elapsed_time = start_time.elapsed();
        let boards_per_second =
            (board_counts.board_count as f64 / elapsed_time.as_secs_f64()) as u64;
        println!(
            "in: {elapsed_time:7.3?}, possibilities: {:12}, calculated: {:12} hz",
            board_counts.board_count.to_formatted_string(&Locale::en),
            boards_per_second.to_formatted_string(&Locale::en)
        );
        board_counts.print_colorfull();
        println!(
            "Average placed ships: {}",
            board_counts.counts.iter().sum::<u64>() as f64 / board_counts.board_count as f64
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
        let (x, y) = (max_index % SIZE, max_index / SIZE);
        (x, y)
    }
    #[rustfmt::skip]
    pub fn inner_loop(&self) -> BoardCounts {
        let mut board_counts = BoardCounts::new();

        let bit_board =BitBoard::new(self.current_board);
        step_summing(bit_board, &mut board_counts, &self.placed_bit_ships);
        board_counts
    }
}

#[inline(always)]
fn step_inner<const INDEX: usize, const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT]>(
    board: BitBoard<SHOULD_PLACE_SHIP>,
    counts: &mut CellCounts<SHOULD_PLACE_SHIP>,
    placed_bit_ships: &PlacedBitShips<SHOULD_PLACE_SHIP>,
) -> u64
where
    [(); count_ships_to_place(SHOULD_PLACE_SHIP)]:,
{
    if !SHOULD_PLACE_SHIP[INDEX] {
        return step_inner_dispatch::<INDEX, SHOULD_PLACE_SHIP>(board, counts, placed_bit_ships);
    }

    let already_placed_ships = const { ship_count_to_ship_index(INDEX, SHOULD_PLACE_SHIP) };
    // directly add the the positions of all possible placements of the last ship
    if const { remaining_ships_test(INDEX, SHOULD_PLACE_SHIP) } == 1 {
        let possible_ship_positions = counts.counts_per_position[already_placed_ships]
            .add_possible_ship_positions(board.allowable::<INDEX>());
        return possible_ship_positions;
    }

    let mut configurations = 0;
    // if const { remaining_ships(SHIP.index(), NEXT_SHIP_INDEX) } == 1 {

    //     for pos_chunk in ChunkedBitIter::new(board.allowable::<SHIP>()) {
    //         let allowable =
    //             pos_chunk.map(|pos| board.place_ship::<SHIP>(pos, placed_bit_ships).allowable::< {Ship::new(2, 0) } >());

    //         let additional_configurations =
    //             counts[Ship::new(2, 0).index].add_possible_ship_positions_chunked(allowable);

    //         for i in 0..ChunkedBitIter::CHUNK_SIZE {
    //             counts[SHIP.index].add_single_ship(pos_chunk[i], additional_configurations[i]);
    //         }
    //         configurations += additional_configurations.iter().sum::<u64>();
    //     }
    //     // counts[NEXT_SHIP_INDEX[SHIP.index]].sum_single_bits();
    //     counts[NEXT_SHIP_INDEX[SHIP.index]].sum_bit_counts(7..8);
    //     return configurations;
    // }

    // for ship_pos in BitIter::new(board.allowable_dyn(ship)) {
    for ship_pos in BitIter::new(board.allowable::<INDEX>()) {
        let board = board.place_ship::<INDEX>(ship_pos, placed_bit_ships);

        let additional_configurations =
            step_inner_dispatch::<INDEX, SHOULD_PLACE_SHIP>(board, counts, placed_bit_ships);
        // addes the currently placed ship with the amount of differnt configurations
        counts.counts_per_position[already_placed_ships]
            .add_single_ship(ship_pos, additional_configurations);

        configurations += additional_configurations;
    }
    // flush the small count with the u8 to the big u64 nums to prevent overflow
    if const { remaining_ships_test(INDEX, SHOULD_PLACE_SHIP) } == 2 {
        counts.sum_last_ship();
    }
    configurations
}

#[rustfmt::skip]
fn step_inner_dispatch<
    const INDEX: usize,
    const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT],
>(
    board: BitBoard<SHOULD_PLACE_SHIP>,
    counts: &mut CellCounts<SHOULD_PLACE_SHIP>,
    placed_bit_ships: &PlacedBitShips<SHOULD_PLACE_SHIP>,
) -> u64
where [(); count_ships_to_place(SHOULD_PLACE_SHIP)]:{
    match INDEX {
        0 => step_inner::<1, SHOULD_PLACE_SHIP>(board,  counts, placed_bit_ships),
        1 => step_inner::<2, SHOULD_PLACE_SHIP>(board,  counts, placed_bit_ships),
        2 => step_inner::<3, SHOULD_PLACE_SHIP>(board,  counts, placed_bit_ships),
        3 => step_inner::<4, SHOULD_PLACE_SHIP>(board,  counts, placed_bit_ships),
        4 => step_inner::<5, SHOULD_PLACE_SHIP>(board,  counts, placed_bit_ships),
        _ => unreachable!()
    }
}
const fn remaining_ships_test(start_index: usize, should_place_ship: [bool; SHIP_COUNT]) -> usize {
    let mut ship_count = 0;
    let mut ship_index = start_index;
    while ship_index < SHIP_COUNT {
        if should_place_ship[ship_index] {
            ship_count += 1;
        }
        ship_index += 1;
    }
    ship_count
}
const fn last_ship_to_place(should_place_ship: [bool; SHIP_COUNT]) -> Ship {
    let mut i = 0;
    let mut max_ship = 0;
    while i < SHIP_COUNT {
        if should_place_ship[i] {
            max_ship = i;
        }
        i += 1;
    }
    SHIPS[max_ship]
}

#[inline(never)]
pub fn step_summing<const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT]>(
    bit_board: BitBoard<SHOULD_PLACE_SHIP>,
    board_counts: &mut BoardCounts,
    placed_bit_ships: &PlacedBitShips<SHOULD_PLACE_SHIP>,
) where
    [(); count_ships_to_place(SHOULD_PLACE_SHIP)]:,
{
    let mut counts = CellCounts::new();
    let total_boards =
        step_inner::<0, { SHOULD_PLACE_SHIP }>(bit_board, &mut counts, placed_bit_ships);

    // checks if only one ship is placed and sums the last placed ship. It otherwise only happens when placing atleast two ships.
    if SHOULD_PLACE_SHIP.iter().filter(|x| **x).count() == 1 {
        counts.sum_last_ship();
    }

    board_counts.board_count += total_boards;
    board_counts.add_cell_counts(counts)
}

// pub fn step<const SHOULD_PLACE_SHIP: [bool; SHIP_COUNT]>(
//     bit_board: BitBoard<SHOULD_PLACE_SHIP>,
//     ship_counts: [u8; 5],
//     board_counts: &mut BoardCounts,
//     placed_bit_ships: &PlacedBitShips<SHOULD_PLACE_SHIP>,
// ) {
//     println!("ship counts: {ship_counts:?}");

//     const T: bool = true;
//     const F: bool = false;
//     match ship_counts {
//         [_, 0, 0, 0, 1] => step_summing::<{ [T, F, F, F, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 0, 0, 1, 0] => step_summing::<{ [F, T, F, F, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 0, 1, 0, 0] => step_summing::<{ [F, F, F, T, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 0, 0, 0] => step_summing::<{ [F, F, F, F, T] }>(bit_board, board_counts, placed_bit_ships),

//         [_, 0, 0, 1, 1] => step_summing::<{ [T, T, F, F, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 0, 1, 0, 1] => step_summing::<{ [T, F, F, T, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 0, 1, 1, 0] => step_summing::<{ [F, T, F, T, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 0, 2, 0, 0] => step_summing::<{ [F, F, T, T, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 0, 0, 1] => step_summing::<{ [T, F, F, F, T] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 0, 1, 0] => step_summing::<{ [F, T, F, F, T] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 1, 0, 0] => step_summing::<{ [F, F, F, T, T] }>(bit_board, board_counts, placed_bit_ships),

//         [_, 0, 1, 1, 1] => step_summing::<{ [T, T, F, T, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 0, 2, 0, 1] => step_summing::<{ [T, F, T, T, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 0, 2, 1, 0] => step_summing::<{ [F, T, T, T, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 0, 1, 1] => step_summing::<{ [T, T, F, F, T] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 1, 0, 1] => step_summing::<{ [T, F, F, T, T] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 1, 1, 0] => step_summing::<{ [F, T, F, T, T] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 2, 0, 0] => step_summing::<{ [F, F, T, T, T] }>(bit_board, board_counts, placed_bit_ships),

//         [_, 0, 2, 1, 1] => step_summing::<{ [T, T, T, T, F] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 1, 1, 1] => step_summing::<{ [T, T, F, T, T] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 2, 0, 1] => step_summing::<{ [T, F, T, T, T] }>(bit_board, board_counts, placed_bit_ships),
//         [_, 1, 2, 1, 0] => step_summing::<{ [F, T, T, T, T] }>(bit_board, board_counts, placed_bit_ships),

//         [_, 1, 2, 1, 1] => step_summing::<{ [T, T, T, T, T] }>(bit_board, board_counts, placed_bit_ships),

//         rem => println!("Not implemented: {rem:?}"),
//     }
// }
