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
    board::{Board, ShipPlacement},
    board_counts::{CellHitCount, CellHitCountSum},
    cell::Cell,
    ship::ShipCounts,
};

#[derive(Clone)]
pub enum DynSolverEnum {
    Solver0(Solver<0>),
    Solver1(Solver<1>),
    Solver2(Solver<2>),
    Solver3(Solver<3>),
    Solver4(Solver<4>),
    Solver5(Solver<5>),
}
impl DynSolverEnum {
    pub fn new(ship_counts: ShipCounts, board: &Board) -> DynSolverEnum {
        match ship_counts.total_ships() {
            0 => DynSolverEnum::Solver0(Solver {
                placed_bit_ships: PlacedBitShips::<0>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
                cell_hit_count: Box::new(CellHitCount::new()),
            }),
            1 => DynSolverEnum::Solver1(Solver {
                placed_bit_ships: PlacedBitShips::<1>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
                cell_hit_count: Box::new(CellHitCount::new()),
            }),
            2 => DynSolverEnum::Solver2(Solver {
                placed_bit_ships: PlacedBitShips::<2>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
                cell_hit_count: Box::new(CellHitCount::new()),
            }),
            3 => DynSolverEnum::Solver3(Solver {
                placed_bit_ships: PlacedBitShips::<3>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
                cell_hit_count: Box::new(CellHitCount::new()),
            }),
            4 => DynSolverEnum::Solver4(Solver {
                placed_bit_ships: PlacedBitShips::<4>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
                cell_hit_count: Box::new(CellHitCount::new()),
            }),
            5 => DynSolverEnum::Solver5(Solver {
                placed_bit_ships: PlacedBitShips::<5>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
                cell_hit_count: Box::new(CellHitCount::new()),
            }),
            ship_count => panic!("Shipcount not supported. is {ship_count}, ship: {board}"),
        }
    }
}
#[derive(Clone)]
pub struct DynSolver {
    // pub dyn_solver: DynSolverEnum,
    pub board: Board,
    pub cell_hit_count_sum: CellHitCountSum,
    pub ship_counts: ShipCounts,
    last_shot: IVec2,
}
impl DynSolver {
    pub fn new(ship_counts: ShipCounts, board: Board, last_shot: IVec2) -> Self {
        Self {
            board,
            ship_counts,
            cell_hit_count_sum: CellHitCountSum::new(),
            last_shot,
        }
    }
    fn gen_dyn_solver(&self) -> DynSolverEnum {
        DynSolverEnum::new(self.ship_counts, &self.board)
    }
    /// returns the arrangement count
    #[rustfmt::skip]
    pub fn step_solver(&mut self) {
        match &mut self.gen_dyn_solver() {
            DynSolverEnum::Solver0(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
            DynSolverEnum::Solver1(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
            DynSolverEnum::Solver2(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
            DynSolverEnum::Solver3(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
            DynSolverEnum::Solver4(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
            DynSolverEnum::Solver5(solver) => solver.step(&self.ship_counts, &mut self.cell_hit_count_sum),
        };
    }
    pub fn calculate_arrangements_with_print(&mut self) -> u64 {
        let start_time = std::time::Instant::now();

        let total_arrangements = self.calculate_arrangements(false);
        let elapsed_time = start_time.elapsed();
        let boards_per_second = (total_arrangements as f64 / elapsed_time.as_secs_f64()) as u64;
        println!(
            "in: {elapsed_time:>8.2?}, possibilities: {:>12}, calculated: {:12} hz",
            total_arrangements.to_formatted_string(&num_format::Locale::en),
            boards_per_second.to_formatted_string(&num_format::Locale::en)
        );
        total_arrangements
    }
    fn place_ship(&self, ship_placement: &ShipPlacement) -> Self {
        DynSolver::new(
            self.ship_counts.remove_placed_ship(ship_placement.ship),
            ship_placement.board.clone(),
            ship_placement.pos,
        )
    }
    #[expect(clippy::collapsible_if)]
    pub fn shoot(&self, shot: IVec2) -> Vec<Self> {
        let miss_allowed = true;
        let mut hit_allowed = true;

        assert!(self.board[shot] == Cell::Water);
        let mut hit_board = self.clone();
        hit_board.board[shot] = Cell::ShipHit;
        let hit_placements = hit_board
            .board
            .all_ship_placements_hit_position(&self.ship_counts, shot);

        if self.board.has_ship_hits() {
            if hit_placements.partial_ship_hit_covering.is_empty() {
                hit_allowed = hit_placements.full_ship_hit_covering.is_empty();
            }
            // let own_ship_placments = self.board.all_ship_placements(&self.ship_counts);
            // if !own_ship_placments.partial_ship_hit_covering.is_empty() {
            //     miss_allowed = own_ship_placments
            //         .partial_ship_hit_covering
            //         .iter()
            //         .any(|placment| placment.board[shoot] == Cell::Protected);
            // }
        }

        let full_ship_hit_coverings = hit_placements
            .full_ship_hit_covering
            .into_iter()
            .filter(|ship_placement| ship_placement.contains_shot(shot))
            .map(|ship_placement| self.place_ship(&ship_placement));

        let mut miss_board = self.clone();
        miss_board.board[shot] = Cell::Protected;

        // let mut sub_solver =
        let mut sub_solvers = Vec::with_capacity(2);
        if miss_allowed {
            sub_solvers.push(DynSolver::new(
                miss_board.ship_counts,
                miss_board.board.clone(),
                shot,
            ));
        }
        if hit_allowed {
            sub_solvers.push(DynSolver::new(
                hit_board.ship_counts,
                hit_board.board.clone(),
                shot,
            ));
        }
        sub_solvers.extend(full_ship_hit_coverings);

        // sub_solvers.retain(|solver| solver.board.is_possible(&solver.ship_counts));

        sub_solvers
    }

    pub fn calculate_arrangements(&mut self, print: bool) -> u64 {
        if print {
            println!("Board: {}", self.board);
        }
        if !self.board.has_ship_hits() {
            self.step_solver();
            return self.cell_hit_count_sum.arrangements;
        }

        let first_hit_index = self
            .board
            .cells
            .iter()
            .position(|cell| *cell == Cell::ShipHit)
            .unwrap();
        let firt_hit_pos = IVec2::new(
            (first_hit_index % SIZE) as i32,
            (first_hit_index / SIZE) as i32,
        );
        let all_ship_placements = self
            .board
            .all_ship_placements_hit_position(&self.ship_counts, firt_hit_pos);
        if print {
            dbg!(all_ship_placements.partial_ship_hit_covering.len());
            dbg!(all_ship_placements.full_ship_hit_covering.len());
        }
        for ship_placement in all_ship_placements.partial_ship_hit_covering {
            if !ship_placement.contains_shot(firt_hit_pos) {
                continue;
            }

            if ship_placement.board.has_ship_hits() {
                let mut solver = self.place_ship(&ship_placement);
                solver.calculate_arrangements(print);
                self.cell_hit_count_sum.add(&solver.cell_hit_count_sum);
                self.cell_hit_count_sum.add_single_placment(
                    ship_placement.clone(),
                    solver.cell_hit_count_sum.arrangements,
                );

                continue;
            }
            let mut solver = self.place_ship(&ship_placement);
            solver.step_solver();
            self.cell_hit_count_sum.add(&solver.cell_hit_count_sum);
            self.cell_hit_count_sum.add_single_placment(
                ship_placement.clone(),
                solver.cell_hit_count_sum.arrangements,
            );
        }
        self.cell_hit_count_sum.arrangements
    }

    pub fn get_best_cell(&self) -> Option<IVec2> {
        let max_index = self
            .board
            .cells
            .iter()
            .zip(&self.cell_hit_count_sum.counts)
            .enumerate()
            .filter(|(_, (cell, _))| **cell == Cell::Water)
            .max_by_key(|(_, (_, count))| **count)
            .unwrap()
            .0;
        if max_index >= 100 {
            // println!("{}", self.current_board);
            // println!("{:?}", self.cell_hit_count_sum.counts);
            return None;
        }

        assert!(max_index < 100, "Index was: {max_index}");
        Some(IVec2::new(
            (max_index % SIZE) as i32,
            (max_index / SIZE) as i32,
        ))
        // IVec2::new((max_index % SIZE) as i32, (max_index / SIZE) as i32)
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
    pub placed_bit_ships: &'static PlacedBitShips<N>,
    bit_board: BitBoard<N>,
    pub cell_hit_count: Box<CellHitCount<N>>,
}

impl<const N: usize> Solver<N> {
    /// returns the count of arrangement count
    fn step(&mut self, ship_counts: &ShipCounts, cell_count_hit_sum: &mut CellHitCountSum) {
        if N == 0 {
            cell_count_hit_sum.arrangements += 1;
            // println!("empty");
            // when there are no ships left there is only one possibility the current one
            return;
        }
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
            let board = board.place_ship::<INDEX>(ship_pos, self.placed_bit_ships);

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
