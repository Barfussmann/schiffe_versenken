use crossterm::event;
use ratatui::DefaultTerminal;

use crate::solver::Solver;

pub struct SolverRender {
    pub solver: Solver<5>,
    pub exit: bool,
}
impl SolverRender {
    pub fn new(solver: Solver<5>) -> Self {
        Self {
            solver,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
        while !self.exit {
            self.solver.run();
            terminal.draw(|frame| self.draw(frame)).unwrap();
            self.handle_events()
        }
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(&self.solver.board_counts, frame.area());
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
