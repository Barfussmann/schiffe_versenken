use rand::Rng;
use rand::thread_rng;

use crate::Ship;

use super::SIZE;

use std::fmt::Display;
use std::fmt::Write;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Direction {
    Horizontal,
    Vetrical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cell {
    Ship,
    Protected,
    Water,
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Cell::Ship => " X ",
            Cell::Protected => " o ",
            Cell::Water => " _ ",
        };
        f.write_str(str)
    }
}

#[derive(Debug)]
pub(crate) enum WasShipplacmentSuccsessfull {
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub(crate) struct Board {
    pub(crate) cells: [Cell; SIZE * SIZE],
}

impl Board {
    pub(crate) fn new() -> Board {
        Board {
            cells: [Cell::Water; SIZE * SIZE],
        }
    }
    pub(crate) fn place_ship(
        &mut self,
        mut x: usize,
        mut y: usize,
        direction: Direction,
        ship: Ship,
    ) -> WasShipplacmentSuccsessfull {
        match direction {
            Direction::Horizontal => {
                assert!(x <= SIZE - ship.length);
                assert!(y < SIZE);
            }
            Direction::Vetrical => {
                assert!(x < SIZE);
                assert!(y <= SIZE - ship.length);
            }
        }

        if !self.is_ship_allowed(x, y, direction, ship) {
            return WasShipplacmentSuccsessfull::No;
        }

        let width;
        let height;
        match direction {
            Direction::Horizontal => {
                width = ship.length + 2;
                height = 3;
            }
            Direction::Vetrical => {
                width = 3;
                height = ship.length + 2;
            }
        };
        self.set_protected_rectangle(x, y, width, height);

        for _ in 0..ship.length {
            let index = Self::cell_index(x, y);
            self.cells[index] = Cell::Ship;

            match direction {
                Direction::Horizontal => x += 1,
                Direction::Vetrical => y += 1,
            }
        }

        WasShipplacmentSuccsessfull::Yes
    }
    pub(crate) fn is_ship_allowed(
        &self,
        mut x: usize,
        mut y: usize,
        direction: Direction,
        ship: Ship,
    ) -> bool {
        for _ in 0..ship.length {
            let index = Self::cell_index(x, y);

            if self.cells[index] != Cell::Water {
                return false;
            }

            match direction {
                Direction::Horizontal => x += 1,
                Direction::Vetrical => y += 1,
            }
        }
        true
    }
    pub(crate) fn set_protected_rectangle(
        &mut self,
        ship_left_x: usize,
        ship_top_y: usize,
        mut width: usize,
        mut height: usize,
    ) {
        if ship_left_x == 0 {
            width -= 1;
        }
        if ship_top_y == 0 {
            height -= 1;
        }

        let low_x = ship_left_x.saturating_sub(1);
        let low_y = ship_top_y.saturating_sub(1);

        let high_x = (low_x + width).min(SIZE);
        let high_y = (low_y + height).min(SIZE);

        for y in low_y..high_y {
            for x in low_x..high_x {
                let index = Self::cell_index(x, y);
                self.cells[index] = Cell::Protected;
            }
        }
    }
    pub(crate) fn cell_index(x: usize, y: usize) -> usize {
        x + y * SIZE
    }

    pub(crate) fn random_place_ship(&mut self, ship: Ship) {
        let rng = &mut thread_rng();

        loop {
            let direction;
            let x;
            let y;
            match rng.gen_bool(0.5) {
                true => {
                    direction = Direction::Horizontal;
                    x = rng.gen_range(0..=SIZE - ship.length);
                    y = rng.gen_range(0..SIZE);
                }
                false => {
                    direction = Direction::Vetrical;
                    x = rng.gen_range(0..SIZE);
                    y = rng.gen_range(0..=SIZE - ship.length);
                }
            };

            // println!("x: {x}, y: {y}, direction: {:?}", direction);
            let was_succsessfull = self.place_ship(x, y, direction, ship);
            match was_succsessfull {
                WasShipplacmentSuccsessfull::Yes => return,
                WasShipplacmentSuccsessfull::No => {}
            }
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        for row in self.cells.chunks(SIZE) {
            for cell in row {
                f.write_fmt(format_args!("{}", cell))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
