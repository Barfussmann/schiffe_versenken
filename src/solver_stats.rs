use std::sync::atomic::{AtomicU64, Ordering};

use ratatui::{
    prelude::*,
    style::Stylize,
    symbols::{Marker, border},
    widgets::{Axis, Block, Chart, Dataset, GraphType},
};

use crate::{CELL_COUNT, solver::DynSolver};

pub struct SolverStats {
    pub games_count_by_depth: [AtomicU64; CELL_COUNT],
    pub solver_count_by_depth: [AtomicU64; CELL_COUNT],
    pub depth: usize,
}

impl SolverStats {
    pub fn new(depth: usize) -> Self {
        SolverStats {
            games_count_by_depth: [const { AtomicU64::new(0) }; CELL_COUNT],
            solver_count_by_depth: [const { AtomicU64::new(0) }; CELL_COUNT],
            depth,
        }
    }
    pub fn add_solver(&self, solver: &DynSolver) {
        self.solver_count_by_depth[solver.depth()].fetch_add(1, Ordering::Relaxed);
        self.games_count_by_depth[solver.depth()]
            .fetch_add(solver.cell_hit_count_sum.arrangements, Ordering::Relaxed);
    }
}

impl Widget for &SolverStats {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Game Lengts".bold());
        let instructions = Line::from("Press 'q' to quit".bold());
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let mut data = Vec::new();
        let mut max_y = 0u64;
        for i in self.depth..self.depth + 50 {
            let game_with_length_i = self.games_count_by_depth[i.saturating_sub(1)]
                .load(Ordering::Relaxed)
                - self.games_count_by_depth[i].load(Ordering::Relaxed);
            data.push((i as f64, game_with_length_i as f64));
            if game_with_length_i > max_y {
                max_y = game_with_length_i;
            }
        }
        max_y = 2u64.pow(max_y.ilog2() + 1);

        let x_axis = Axis::default()
            .title("X Axis".red())
            .style(Style::default().white())
            .bounds([0.0, 60.0])
            .labels(["0", "10", "20", "30", "40", "50"]);

        let y_axis = Axis::default()
            .title("Y Axis".red())
            .style(Style::default().white())
            .bounds([0.0, max_y as f64])
            .labels(["0".to_string(), format!("{max_y}")]);

        let dataset = Dataset::default()
            .name("Game Length")
            .marker(Marker::HalfBlock)
            .graph_type(GraphType::Bar)
            .style(Style::default().green())
            .data(&data);

        Chart::new(vec![dataset])
            .block(block)
            .x_axis(x_axis)
            .y_axis(y_axis)
            .render(area, buf);
    }
}
