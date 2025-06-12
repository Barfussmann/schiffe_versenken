use num_format::{Locale, ToFormattedString};

use crate::{
    SIZE,
    bit_board::{BitBoard, PlacedBitShips},
    bit_iter::BitIter,
    board::Board,
    board_counts::BoardCounts,
    cell::Cell,
    ship::ShipCounts,
};

pub enum DynSolverEnum {
    Solver1(Solver<1>),
    Solver2(Solver<2>),
    Solver3(Solver<3>),
    Solver4(Solver<4>),
    Solver5(Solver<5>),
}
pub struct DynSolver {
    pub dyn_solver: DynSolverEnum,
    pub current_board: Board,
    pub ship_counts: ShipCounts,
    pub board_counts: u64,
}
impl DynSolver {
    pub fn new(ship_counts: ShipCounts, board: Board) -> Self {
        let dyn_solver = match ship_counts.total_ship_count() {
            1 => DynSolverEnum::Solver1(Solver::new(ship_counts, &board)),
            2 => DynSolverEnum::Solver2(Solver::new(ship_counts, &board)),
            3 => DynSolverEnum::Solver3(Solver::new(ship_counts, &board)),
            4 => DynSolverEnum::Solver4(Solver::new(ship_counts, &board)),
            5 => DynSolverEnum::Solver5(Solver::new(ship_counts, &board)),
            count => unreachable!("Only 1 to 5 Ships are allowed. Is {count}"),
        };
        Self {
            dyn_solver,
            current_board: board,
            ship_counts,
            board_counts: 0,
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new(self.ship_counts, Board::new());
    }
    pub fn step(&mut self) {
        let start_time = std::time::Instant::now();

        self.board_counts = match &mut self.dyn_solver {
            DynSolverEnum::Solver1(solver) => solver.step(&self.ship_counts),
            DynSolverEnum::Solver2(solver) => solver.step(&self.ship_counts),
            DynSolverEnum::Solver3(solver) => solver.step(&self.ship_counts),
            DynSolverEnum::Solver4(solver) => solver.step(&self.ship_counts),
            DynSolverEnum::Solver5(solver) => solver.step(&self.ship_counts),
        };
        let elapsed_time = start_time.elapsed();
        let boards_per_second = (self.board_counts as f64 / elapsed_time.as_secs_f64()) as u64;
        println!(
            "in: {elapsed_time:8.2?}, possibilities: {:>12}, calculated: {:12} hz",
            self.board_counts
                .to_formatted_string(&num_format::Locale::en),
            boards_per_second.to_formatted_string(&num_format::Locale::en)
        );
    }
    // pub fn run(&mut self) {}
    pub fn split_and_count(&mut self) {
        self.current_board.cells[44] = Cell::ShipHit;
        *self = DynSolver::new(self.ship_counts, self.current_board.clone());

        self.step();
        let mut total = self.board_counts;

        for (sub_board, placed_ship) in self
            .current_board
            .all_ship_placements(&self.ship_counts)
            .partial_ship_hit_covering
        {
            println!("length: {}", placed_ship.length());
            let new_ship_counts = self.ship_counts.remove_placed_ship(placed_ship);
            let mut solver = DynSolver::new(new_ship_counts, sub_board);
            solver.step();
            let sub_count = solver.board_counts;
            total += sub_count;
        }
        println!("total: {}", total.to_formatted_string(&Locale::en));
        self.reset();
        self.step();
    }
}

pub struct Solver<const N: usize> {
    pub placed_bit_ships: Box<PlacedBitShips<N>>,
    bit_board: BitBoard<N>,
    pub board_counts: Box<BoardCounts<N>>,
}

impl<const N: usize> Solver<N> {
    pub fn new(ship_counts: ShipCounts, board: &Board) -> Self {
        Solver {
            placed_bit_ships: Box::new(PlacedBitShips::new(ship_counts)),
            bit_board: board.to_bitboard(ship_counts),
            board_counts: Box::new(BoardCounts::new()),
        }
    }
    /// returns the count of different boards
    fn step(&mut self, ship_counts: &ShipCounts) -> u64 {
        let board_count = self.place_ship_recursive::<0>(self.bit_board);
        self.board_counts.sum_cell_counts(ship_counts);
        board_count as u64
    }
    pub fn get_best_cell(&mut self, ship_counts: &ShipCounts) {
        let start_time = std::time::Instant::now();
        self.step(ship_counts);

        let (x, y) = self.get_best_water_cell();

        let elapsed_time = start_time.elapsed();
        let boards_per_second =
            (self.board_counts.board_count as f64 / elapsed_time.as_secs_f64()) as u64;
        println!(
            "in: {elapsed_time:7.3?}, possibilities: {:12}, calculated: {:12} hz",
            self.board_counts
                .board_count
                .to_formatted_string(&num_format::Locale::en),
            boards_per_second.to_formatted_string(&num_format::Locale::en)
        );
        println!(
            "Average placed ships: {}",
            self.board_counts.counts.iter().sum::<u32>() as f64
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
        // assert!(self.current_board.cells[max_index] == Cell::Water); ToDo reimplement this
        let (x, y) = (max_index % SIZE, max_index / SIZE);
        (x, y)
    }

    #[inline(always)]
    fn place_ship_recursive<const INDEX: usize>(&mut self, board: BitBoard<N>) -> u32 {
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
