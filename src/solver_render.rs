use crossterm::event;
use ratatui::DefaultTerminal;

use crate::solver::{DynSolver, DynSolverEnum};

pub struct SolverRender {
    pub solver: DynSolver,
    pub exit: bool,
}
impl SolverRender {
    pub fn new(solver: DynSolver) -> Self {
        Self {
            solver,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
        while !self.exit {
            self.solver.step();
            terminal.draw(|frame| self.draw(frame)).unwrap();
            self.handle_events()
        }
    }

    #[rustfmt::skip]
    fn draw(&self, frame: &mut ratatui::Frame) {
        match &self.solver.dyn_solver {
            DynSolverEnum::Solver1(solver) => frame.render_widget(solver.board_counts.as_ref(), frame.area()),
            DynSolverEnum::Solver2(solver) => frame.render_widget(solver.board_counts.as_ref(), frame.area()),
            DynSolverEnum::Solver3(solver) => frame.render_widget(solver.board_counts.as_ref(), frame.area()),
            DynSolverEnum::Solver4(solver) => frame.render_widget(solver.board_counts.as_ref(), frame.area()),
            DynSolverEnum::Solver5(solver) => frame.render_widget(solver.board_counts.as_ref(), frame.area()),
        }
    }

    fn handle_events(&mut self) {
        match event::read().unwrap() {
            event::Event::Key(key_event) if key_event.kind == event::KeyEventKind::Press => {
                match key_event.code {
                    event::KeyCode::Char('q') => self.exit = true,
                    _ => todo!(),
                }
            }
            _ => {}
        }
    }
}
