use std::sync::atomic::{AtomicU64, Ordering};

use crate::{CELL_COUNT, solver::DynSolver};

pub struct SolverStats {
    pub games_count_by_depth: [AtomicU64; CELL_COUNT],
    pub solver_count_by_depth: [AtomicU64; CELL_COUNT],
}

impl SolverStats {
    pub fn new() -> Self {
        SolverStats {
            games_count_by_depth: [const { AtomicU64::new(0) }; CELL_COUNT],
            solver_count_by_depth: [const { AtomicU64::new(0) }; CELL_COUNT],
        }
    }
    pub fn add_solver(&self, solver: &DynSolver) {
        self.solver_count_by_depth[solver.depth()].fetch_add(1, Ordering::Relaxed);
        self.games_count_by_depth[solver.depth()]
            .fetch_add(solver.cell_hit_count_sum.arrangements, Ordering::Relaxed);
    }
}
