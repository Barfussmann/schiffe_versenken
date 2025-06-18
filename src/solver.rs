use colorgrad::Gradient;
use glam::{IVec2, ivec2};
use num_format::ToFormattedString;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize, palette::material::BLACK},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    SIZE,
    bit_board::{BitBoard, PlacedBitShips},
    bit_iter::BitIter,
    board::{Board, ShipPlacement},
    board_counts::{CellHitCount, CellHitCountSum},
    cell::Cell,
    ship::ShipCounts,
    solver_stats::SolverStats,
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
            }),
            1 => DynSolverEnum::Solver1(Solver {
                placed_bit_ships: PlacedBitShips::<1>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
            }),
            2 => DynSolverEnum::Solver2(Solver {
                placed_bit_ships: PlacedBitShips::<2>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
            }),
            3 => DynSolverEnum::Solver3(Solver {
                placed_bit_ships: PlacedBitShips::<3>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
            }),
            4 => DynSolverEnum::Solver4(Solver {
                placed_bit_ships: PlacedBitShips::<4>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
            }),
            5 => DynSolverEnum::Solver5(Solver {
                placed_bit_ships: PlacedBitShips::<5>::new(ship_counts),
                bit_board: board.to_bitboard(ship_counts),
            }),
            ship_count => panic!("Shipcount not supported. is {ship_count}, ship: {board}"),
        }
    }
}
#[derive(Clone)]
pub struct DynSolver {
    pub board: Board,
    pub cell_hit_count_sum: CellHitCountSum,
    pub ship_counts: ShipCounts,
    depth: usize,
}
impl DynSolver {
    pub fn new(ship_counts: ShipCounts, board: Board, depth: usize) -> Self {
        Self {
            board,
            ship_counts,
            cell_hit_count_sum: CellHitCountSum::new(),
            depth,
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
    fn place_ship(&self, ship_placement: &ShipPlacement) -> Self {
        DynSolver::new(
            self.ship_counts.remove_placed_ship(ship_placement.ship),
            ship_placement.board.clone(),
            self.depth + 1,
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
            // if !hit_placements.partial_ship_hit_covering.is_empty() {
            //     miss_allowed = hit_placements
            //         .partial_ship_hit_covering
            //         .iter()
            //         .any(|placment| placment.board[shot] == Cell::Protected);
            // }
        }

        let mut miss_board = self.clone();
        miss_board.board[shot] = Cell::Protected;

        // let mut sub_solver =
        let mut sub_solvers = Vec::with_capacity(2);
        if miss_allowed {
            sub_solvers.push(DynSolver::new(
                miss_board.ship_counts,
                miss_board.board.clone(),
                self.depth() + 1,
            ));
        }
        if hit_allowed {
            sub_solvers.push(DynSolver::new(
                hit_board.ship_counts,
                hit_board.board.clone(),
                self.depth() + 1,
            ));
        }
        sub_solvers.extend(
            hit_placements
                .full_ship_hit_covering
                .into_iter()
                .map(|ship_placement| self.place_ship(&ship_placement)),
        );

        sub_solvers
    }
    pub fn calculate_arrangements_in_depth(&mut self, depth: usize) -> SolverStats {
        let solver_stats = SolverStats::new(self.depth());
        self.calculate_arrangements_in_depth_inner(depth, &solver_stats);
        solver_stats
    }
    pub fn calculate_arrangements_in_depth_inner(
        &mut self,
        depth: usize,
        solver_stats: &SolverStats,
    ) -> u64 {
        self.calculate_arrangements();
        solver_stats.add_solver(self);

        if depth == 0 {
            return self.cell_hit_count_sum.arrangements;
        }
        // if self.ship_counts.total_ships() == 1 {
        //     return self.cell_hit_count_sum.arrangements;
        // }

        let Some(shot) = self.get_best_cell() else {
            // println!("board: {}", self.board);
            return self.cell_hit_count_sum.arrangements;
        };
        let _sub_arrangements = self
            .shoot(shot)
            .into_par_iter()
            .map(|mut solver| solver.calculate_arrangements_in_depth_inner(depth - 1, solver_stats))
            .sum::<u64>();
        // assert_eq!(self.cell_hit_count_sum.arrangements, sub_arrangements);
        self.cell_hit_count_sum.arrangements
    }
    fn iterate_all_ship_placement_sub_solvers(
        self,
        fun: &mut impl FnMut(DynSolver, ShipPlacement),
    ) {
        let first_hit_pos = self.get_first_hit_pos();
        let all_ship_placements = self
            .board
            .all_ship_placements_hit_position(&self.ship_counts, first_hit_pos);
        for ship_placement in all_ship_placements.partial_ship_hit_covering {
            if !ship_placement.board.has_ship_hits() {
                fun(self.place_ship(&ship_placement), ship_placement);
            } else {
                self.place_ship(&ship_placement)
                    .iterate_all_ship_placement_sub_solvers(fun);
            }
        }
    }
    pub fn calculate_arrangements(&mut self) -> u64 {
        if !self.board.has_ship_hits() {
            self.step_solver();
            return self.cell_hit_count_sum.arrangements;
        }

        self.clone().iterate_all_ship_placement_sub_solvers(
            &mut |mut sub_solver, ship_placement| {
                sub_solver.step_solver();
                self.cell_hit_count_sum.add(&sub_solver.cell_hit_count_sum);
                self.cell_hit_count_sum.add_single_placment(
                    ship_placement,
                    sub_solver.cell_hit_count_sum.arrangements,
                );
            },
        );
        self.cell_hit_count_sum.arrangements
    }

    fn get_first_hit_pos(&self) -> IVec2 {
        let first_hit_index = self
            .board
            .cells
            .iter()
            .position(|cell| *cell == Cell::ShipHit)
            .unwrap();
        IVec2::new(
            (first_hit_index % SIZE) as i32,
            (first_hit_index / SIZE) as i32,
        )
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
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
    fn to_ratatui_text(&self) -> Text<'_> {
        let counts = &self.cell_hit_count_sum;
        let max_val = *counts.counts.iter().max().unwrap() as f32;

        let color_grad = colorgrad::preset::rd_yl_gn();

        Text::from_iter(
            counts
                .counts
                .chunks(SIZE)
                .take(SIZE)
                .enumerate()
                .map(|(y, row)| {
                    Line::from_iter(row.iter().enumerate().map(|(x, count)| {
                        let probability = *count as f32 / (counts.arrangements as f32);
                        let color_scale = *count as f32 / max_val;
                        let rgba8 = color_grad.at(color_scale).to_rgba8();

                        let cell = self.board[ivec2(x as i32, y as i32)];

                        match cell {
                            Cell::Water => Span::styled(
                                format_args!("{:3.0}", probability * 1000.).to_string(),
                                Style::default()
                                    .bg(Color::Rgb(rgba8[0], rgba8[1], rgba8[2]))
                                    .fg(BLACK),
                            ),
                            Cell::Protected => {
                                Span::styled(" O ", Style::default().fg(Color::Green))
                            }
                            Cell::ShipHit => Span::styled(" X ", Style::default().fg(Color::Red)),
                            Cell::Ship => Span::styled(" X ", Style::default().fg(Color::Green)),
                        }
                    }))
                }),
        )
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

        let probabilities = self.to_ratatui_text();

        Paragraph::new(probabilities)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[derive(Clone)]
pub struct Solver<const N: usize> {
    pub placed_bit_ships: &'static PlacedBitShips<N>,
    bit_board: BitBoard<N>,
    // pub cell_hit_count: Box<>,
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
        let mut cell_hit_count = CellHitCount::<N>::new();
        cell_count_hit_sum.arrangements =
            self.place_ship_recursive::<0>(self.bit_board, &mut cell_hit_count) as u64;
        cell_hit_count.sum_cell_counts(ship_counts, cell_count_hit_sum);
    }

    #[inline(always)]
    fn place_ship_recursive<const INDEX: usize>(
        &mut self,
        board: BitBoard<N>,
        cell_hit_count: &mut CellHitCount<N>,
    ) -> u32 {
        // directly add the the positions of all possible placements of the last ship
        if INDEX + 1 == N {
            return cell_hit_count
                .ship_cell_counts
                .add_possible_ship_positions(board.allowable::<INDEX>());
            // return self.board_counts.ship_cell_counts.counts_per_ship_position[INDEX]
            //     .add_possible_ship_positions(board.allowable::<INDEX>());
        }

        let mut configurations = 0;
        for ship_pos in BitIter::new(board.allowable::<INDEX>()) {
            let board = board.place_ship::<INDEX>(ship_pos, self.placed_bit_ships);

            let additional_configurations = match INDEX {
                0 => self.place_ship_recursive::<1>(board, cell_hit_count),
                1 => self.place_ship_recursive::<2>(board, cell_hit_count),
                2 => self.place_ship_recursive::<3>(board, cell_hit_count),
                3 => self.place_ship_recursive::<4>(board, cell_hit_count),
                4 => self.place_ship_recursive::<5>(board, cell_hit_count),
                _ => unreachable!(),
            };
            // addes the currently placed ship with the count of sub-configurations
            cell_hit_count.ship_cell_counts.ship_count_per_cell[INDEX]
                .add_single_ship(ship_pos, additional_configurations);

            configurations += additional_configurations;
        }

        // flush the cell counts to prevent overflow
        if INDEX + 2 == N {
            cell_hit_count.ship_cell_counts.sum_last_ship();
        }
        configurations
    }
}
