use glam::IVec2;
use num_format::ToFormattedString;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    SIZE,
    bit_board::{BitBoard, PlacedBitShips},
    bit_iter::BitIter,
    board::Board,
    board_counts::{CellHitCount, CellHitCountSum},
    cell::Cell,
    ship::ShipCounts,
};

#[derive(Clone)]
pub enum DynSolverEnum {
    Solver1(Solver<1>),
    Solver2(Solver<2>),
    Solver3(Solver<3>),
    Solver4(Solver<4>),
    Solver5(Solver<5>),
}
#[derive(Clone)]
pub struct DynSolver {
    // pub dyn_solver: DynSolverEnum,
    pub current_board: Board,
    pub cell_hit_count_sum: CellHitCountSum,
    pub ship_counts: ShipCounts,
}
impl DynSolver {
    pub fn new(ship_counts: ShipCounts, board: Board) -> Self {
        Self {
            current_board: board,
            ship_counts,
            cell_hit_count_sum: CellHitCountSum::new(),
        }
    }
    fn gen_dyn_solver(&self) -> DynSolverEnum {
        match self.ship_counts.total_ship_count() {
            1 => DynSolverEnum::Solver1(Solver::new(self.ship_counts, &self.current_board)),
            2 => DynSolverEnum::Solver2(Solver::new(self.ship_counts, &self.current_board)),
            3 => DynSolverEnum::Solver3(Solver::new(self.ship_counts, &self.current_board)),
            4 => DynSolverEnum::Solver4(Solver::new(self.ship_counts, &self.current_board)),
            5 => DynSolverEnum::Solver5(Solver::new(self.ship_counts, &self.current_board)),
            count => unreachable!("Only 1 to 5 Ships are allowed. Is {count}"),
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new(self.ship_counts, Board::new());
    }
    /// returns the arrangement count
    #[rustfmt::skip]
    pub fn step_solver(&mut self) {
        match &mut self.gen_dyn_solver() {
            DynSolverEnum::Solver1(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
            DynSolverEnum::Solver2(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
            DynSolverEnum::Solver3(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
            DynSolverEnum::Solver4(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
            DynSolverEnum::Solver5(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
        };
    }
    pub fn calculate_arrangements_with_print(&mut self) -> u64 {
        let start_time = std::time::Instant::now();

        let total_arrangements = self.calculate_arrangements();
        let elapsed_time = start_time.elapsed();
        let boards_per_second = (total_arrangements as f64 / elapsed_time.as_secs_f64()) as u64;
        println!(
            "in: {elapsed_time:>8.2?}, possibilities: {:>12}, calculated: {:12} hz",
            total_arrangements.to_formatted_string(&num_format::Locale::en),
            boards_per_second.to_formatted_string(&num_format::Locale::en)
        );
        total_arrangements
    }
    pub fn shoot(&self, shoot: IVec2) -> Vec<Self> {
        // let mut hit_allowed = true;
        let mut miss_allowed = true;
        let own_ship_placments = self.current_board.all_ship_placements(&self.ship_counts);
        if !own_ship_placments.partial_ship_hit_covering.is_empty() {
            miss_allowed = own_ship_placments
                .partial_ship_hit_covering
                .iter()
                .any(|placment| placment.placed_board[shoot] == Cell::Protected);
        }

        assert!(self.current_board[shoot] == Cell::Water);
        let mut hit_board = self.clone();
        hit_board.current_board[shoot] = Cell::ShipHit;

        let full_ship_hit_coverings = hit_board
            .current_board
            .all_ship_placements(&self.ship_counts)
            .full_ship_hit_covering
            .into_iter()
            .map(|ship_placment| {
                DynSolver::new(
                    self.ship_counts.remove_placed_ship(ship_placment.ship),
                    ship_placment.placed_board,
                )
            });

        let mut miss_board = self.clone();
        miss_board.current_board[shoot] = Cell::Protected;

        // let mut sub_solver =
        let mut sub_solvers = vec![DynSolver::new(
            hit_board.ship_counts,
            hit_board.current_board.clone(),
        )];
        if miss_allowed {
            sub_solvers.push(DynSolver::new(
                miss_board.ship_counts,
                miss_board.current_board.clone(),
            ))
        }
        sub_solvers.extend(full_ship_hit_coverings);

        sub_solvers
    }

    pub fn calculate_arrangements(&mut self) -> u64 {
        let all_ship_placements = self.current_board.all_ship_placements(&self.ship_counts);
        if all_ship_placements.partial_ship_hit_covering.is_empty()
            && all_ship_placements.full_ship_hit_covering.is_empty()
        {
            self.step_solver();
        }
        for ship_placement in all_ship_placements.partial_ship_hit_covering {
            for cell in ship_placement.placed_board.cells {
                assert!(cell != Cell::ShipHit);
            }
            let mut solver = DynSolver::new(
                self.ship_counts.remove_placed_ship(ship_placement.ship),
                ship_placement.placed_board.clone(),
            );
            solver.step_solver();
            self.cell_hit_count_sum.add(&solver.cell_hit_count_sum);
            self.cell_hit_count_sum
                .add_single_placment(ship_placement, solver.cell_hit_count_sum.arrangements);
        }
        self.cell_hit_count_sum.arrangements
    }
    pub fn get_best_cell(&self) -> IVec2 {
        let max_index = self
            .current_board
            .cells
            .iter()
            .zip(&self.cell_hit_count_sum.counts)
            .enumerate()
            .filter(|(_, (cell, _))| **cell == Cell::Water)
            .max_by_key(|(_, (_, count))| **count)
            .unwrap()
            .0;
        assert!(max_index < 100, "Index was: {max_index}");
        IVec2::new((max_index % SIZE) as i32, (max_index / SIZE) as i32)
    }
}
impl Widget for &DynSolver {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Ship hit probabilities".bold());
        let instructions = Line::from("Press 'q' to quit".bold());
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let probs = self.cell_hit_count_sum.to_ratatui_text();

        Paragraph::new(probs)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[derive(Clone)]
pub struct Solver<const N: usize> {
    pub placed_bit_ships: Box<PlacedBitShips<N>>,
    bit_board: BitBoard<N>,
    pub cell_hit_count: Box<CellHitCount<N>>,
}

impl<const N: usize> Solver<N> {
    pub fn new(ship_counts: ShipCounts, board: &Board) -> Self {
        Solver {
            placed_bit_ships: Box::new(PlacedBitShips::new(ship_counts)),
            bit_board: board.to_bitboard(ship_counts),
            cell_hit_count: Box::new(CellHitCount::new()),
        }
    }
    /// returns the count of arrangement count
    fn step(&mut self, ship_counts: &ShipCounts, cell_count_hit_sum: &mut CellHitCountSum) {
        cell_count_hit_sum.arrangements = self.place_ship_recursive::<0>(self.bit_board) as u64;
        self.cell_hit_count
            .sum_cell_counts(ship_counts, cell_count_hit_sum);
    }

    #[inline(always)]
    fn place_ship_recursive<const INDEX: usize>(&mut self, board: BitBoard<N>) -> u32 {
        // directly add the the positions of all possible placements of the last ship
        if INDEX + 1 == N {
            return self
                .cell_hit_count
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
            self.cell_hit_count.ship_cell_counts.ship_count_per_cell[INDEX]
                .add_single_ship(ship_pos, additional_configurations);

            configurations += additional_configurations;
        }

        // flush the cell counts to prevent overflow
        if INDEX + 2 == N {
            self.cell_hit_count.ship_cell_counts.sum_last_ship();
        }
        configurations
    }
}
