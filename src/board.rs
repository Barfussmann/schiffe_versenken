use crate::Ship;
use crate::ShipCounts;
use crate::{BOARD_SIZE, SHIPS, SIZE};

const PLACED_SHIPS: [[[[Board; SIZE]; SIZE];2]; 10]


use std::fmt::Display;
use std::fmt::Write;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vetrical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
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
pub enum WasShipplacmentSuccsessfull {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy)]
#[repr(align(128))]
pub struct Board {
    pub cells: [Cell; BOARD_SIZE],
}

impl Board {
    pub fn new() -> Board {
        Board {
            cells: [Cell::Water; BOARD_SIZE],
        }
    }
    pub fn place_ship(
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
    pub fn is_ship_allowed(
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
    pub fn set_protected_rectangle(
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
    pub fn cell_index(x: usize, y: usize) -> usize {
        x + y * SIZE
    }
    pub fn place_all_ships_recursive(
        &self,
        start_x: usize,
        start_y: usize,
        ships: &[Ship],
        ship_counts: &mut ShipCounts,
    ) {
        if ships.is_empty() {
            ship_counts.add_board(*self);
            return;
        }
        let (ship_to_place, remaining_ships) = ships.split_first().unwrap();
        let next_ship_same = ship_to_place == remaining_ships.first().unwrap_or(&SHIPS[0]);

        let mut valid_boards = 0;
        for direction in [Direction::Horizontal, Direction::Vetrical] {
            let max_x;
            let max_y;
            match direction {
                Direction::Horizontal => {
                    max_x = SIZE - (ship_to_place.length - 1); // have to reduce by one because 1 length ships also need to be placed at the cornern
                    max_y = SIZE;
                }
                Direction::Vetrical => {
                    max_x = SIZE;
                    max_y = SIZE - (ship_to_place.length - 1); // have to reduce by one because 1 length ships also need to be placed at the cornern
                }
            }
            for y in start_x..max_y {
                for x in start_y..max_x {
                    let mut me = *self;
                    match me.place_ship(x, y, direction, *ship_to_place) {
                        WasShipplacmentSuccsessfull::Yes => {
                            if next_ship_same {
                                me.place_all_ships_recursive(x, y, remaining_ships, ship_counts);
                            } else {
                                me.place_all_ships_recursive(0, 0, remaining_ships, ship_counts);
                            }

                            valid_boards += 1;
                        }
                        WasShipplacmentSuccsessfull::No => {}
                    };
                    // me.place_all_ships_recursive(remaining_ships, ship_counts);
                }
            }
        }
        if ships.len() >= 6 {
            println!("{}", self);
            println!("{}", ships.len());
            println!("{}", ship_counts.board_count);
            println!("{}", valid_boards);
        }
    }
    pub fn place_all_ships_random_recursive(
        &mut self,
        ships: &[Ship],
        ship_counts: &mut ShipCounts,
        rng: &mut fastrand::Rng,
    ) {
        if ships.is_empty() {
            ship_counts.add_board(*self);
            return;
        }
        let (ship_to_place, remaining_ships) = ships.split_first().unwrap();

        let mut me = *self;
        match me.try_radom_place_ship(*ship_to_place, rng) {
            WasShipplacmentSuccsessfull::Yes => {
                me.place_all_ships_random_recursive(remaining_ships, ship_counts, rng)
            }
            WasShipplacmentSuccsessfull::No => {}
        }

        match self.try_radom_place_ship(*ship_to_place, rng) {
            WasShipplacmentSuccsessfull::Yes => {
                self.place_all_ships_random_recursive(remaining_ships, ship_counts, rng)
            }
            WasShipplacmentSuccsessfull::No => {}
        };
    }
    pub fn try_radom_place_ship(
        &mut self,
        ship: Ship,
        rng: &mut fastrand::Rng,
    ) -> WasShipplacmentSuccsessfull {
        let direction;
        let x;
        let y;

        match rng.bool() {
            true => {
                direction = Direction::Horizontal;
                x = rng.u8(0..=(SIZE - ship.length) as u8) as usize;
                y = rng.u8(0..(SIZE) as u8) as usize;
            }
            false => {
                direction = Direction::Vetrical;
                x = rng.u8(0..SIZE as u8) as usize;
                y = rng.u8(0..=(SIZE - ship.length) as u8) as usize;
            }
        };

        self.place_ship(x, y, direction, ship)
    }
    pub fn random_place_ship(&mut self, ship: Ship, rng: &mut fastrand::Rng) {
        while let WasShipplacmentSuccsessfull::No = self.try_radom_place_ship(ship, rng) {}
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        for row in self.cells.chunks(SIZE).take(SIZE) {
            for cell in row {
                f.write_fmt(format_args!("{}", cell))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
