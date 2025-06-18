use crossterm::event;
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
};

use crate::{solver::DynSolver, solver_stats::SolverStats};

pub struct SolverRender {
    solver: DynSolver,
    exit: bool,
    solver_stats: Option<SolverStats>,
}
impl SolverRender {
    pub fn new(mut solver: DynSolver) -> Self {
        solver.calculate_arrangements();
        Self {
            solver,
            exit: false,
            solver_stats: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
        while !self.exit {
            // self.solver.calculate_arrangements();
            self.solver_stats = Some(self.solver.calculate_arrangements_in_depth(70));
            terminal.draw(|frame| self.draw(frame)).unwrap();
            self.handle_events();
        }
    }

    #[rustfmt::skip]
    fn draw(&self, frame: &mut ratatui::Frame) {


        let horizontal = Layout::horizontal([Constraint::Length(32), Constraint::Fill(1)]);
        let [left, right] = horizontal.areas::<2>(frame.area())[..].try_into().unwrap();
        frame.render_widget(&self.solver, left);

        if let Some(stats) = &self.solver_stats {
            frame.render_widget(stats, right);
        }
    }

    fn handle_events(&mut self) {
        match event::read().unwrap() {
            event::Event::Key(key_event) if key_event.kind == event::KeyEventKind::Press => {
                match key_event.code {
                    event::KeyCode::Char('q') => self.exit = true,
                    event::KeyCode::Char('u') => {
                        self.solver =
                            self.solver.shoot(self.solver.get_best_cell().unwrap())[0].clone();
                        self.solver.calculate_arrangements();
                    }
                    event::KeyCode::Char('i') => {
                        self.solver =
                            self.solver.shoot(self.solver.get_best_cell().unwrap())[1].clone();
                        self.solver.calculate_arrangements();
                    }
                    event::KeyCode::Char('a') => {
                        self.solver =
                            self.solver.shoot(self.solver.get_best_cell().unwrap())[2].clone();
                        self.solver.calculate_arrangements();
                    }
                    _ => (),
                }
            }
            _ => {}
        }
    }
}
