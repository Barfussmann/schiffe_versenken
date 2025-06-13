use crossterm::event;
use ratatui::DefaultTerminal;

use crate::solver::DynSolver;

pub struct SolverRender {
    pub solver: DynSolver,

    pub exit: bool,
}
impl SolverRender {
    pub fn new(mut solver: DynSolver) -> Self {
        solver.calculate_arrangements();
        Self {
            solver,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame)).unwrap();
            self.handle_events()
        }
    }

    #[rustfmt::skip]
    fn draw(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(&self.solver, frame.area());
    }

    fn handle_events(&mut self) {
        match event::read().unwrap() {
            event::Event::Key(key_event) if key_event.kind == event::KeyEventKind::Press => {
                match key_event.code {
                    event::KeyCode::Char('q') => self.exit = true,
                    event::KeyCode::Char('u') => {
                        self.solver.calculate_arrangements();
                    }
                    event::KeyCode::Char('i') => {
                        self.solver = self.solver.shoot(self.solver.get_best_cell())[0].clone();
                        self.solver.calculate_arrangements();
                    }
                    event::KeyCode::Char('a') => {
                        self.solver = self.solver.shoot(self.solver.get_best_cell())[1].clone();
                        self.solver.calculate_arrangements();
                    }
                    event::KeyCode::Char('e') => {
                        self.solver = self.solver.shoot(self.solver.get_best_cell())[2].clone();
                        self.solver.calculate_arrangements();
                    }
                    _ => (),
                }
            }
            _ => {}
        }
    }
}
