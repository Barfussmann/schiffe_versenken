use crate::Ship;
use crate::cell_grid::CellGrid;
use crate::ship_counts::ShipCounts;
use crate::{SHIPS, SIZE};

static PLACED_SHIPS: [[[[Board; SIZE]; SIZE]; 2]; 10] = {
    let mut placed_ships = [[[[Board::new(Cell::Water); SIZE]; SIZE]; 2]; 10];

    let mut ship_i = 0;
    while ship_i < SHIPS.len() {
        let mut dir_i = 0;
        while dir_i < 2 {
            let mut y = 0;
            while y < SIZE {
                let mut x = 0;
                while x < SIZE {
                    let ship = SHIPS[ship_i];

                    let dir = match dir_i {
                        0 => Direction::Horizontal,
                        1 => Direction::Vetrical,
                        _ => unreachable!(),
                    };

                    placed_ships[ship_i][dir_i][y][x].const_place_ship(x, y, dir, ship);

                    x += 1;
                }
                y += 1;
            }
            dir_i += 1;
        }
        ship_i += 1;
    }

    placed_ships
};

use std::fmt::Display;
use std::fmt::Write;
use std::iter::zip;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal = 0,
    Vetrical = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cell {
    Water = 0,
    Protected = 1,
    ShipHit = 2,
    Ship = 3,
}
impl Cell {
    pub fn protect(&mut self) {
        match self {
            Cell::Water => *self = Cell::Protected,
            Cell::Protected | Cell::ShipHit | Cell::Ship => {}
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Cell::Ship => " X ",
            Cell::Protected => " o ",
            Cell::Water => " _ ",
            Cell::ShipHit => " X ",
        };
        f.write_str(str)
    }
}

#[derive(Debug)]
pub enum WasShipplacmentSuccsessfull {
    Yes,
    No,
}

pub type Board = CellGrid<Cell>;

impl Board {
    const fn saturating_cell_index(mut x: usize, mut y: usize) -> usize {
        if x >= SIZE {
            x = SIZE - 1;
        }
        if y >= SIZE {
            y = SIZE - 1;
        }

        Self::cell_index(x, y)
    }

    pub const fn const_place_ship(
        &mut self,
        mut x: usize,
        mut y: usize,
        direction: Direction,
        ship: Ship,
    ) {
        let mut width;
        let mut height;
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
        if x == 0 {
            width -= 1;
        }
        if y == 0 {
            height -= 1;
        }

        let low_x = x.saturating_sub(1);
        let low_y = y.saturating_sub(1);

        let high_x = low_x + width;
        let high_y = low_y + height;

        let mut i_y = low_y;
        while i_y < high_y {
            let mut i_x = low_x;
            while i_x < high_x {
                if self.is_inside(i_x, i_y) {
                    self.cells[i_y][i_x] = Cell::Protected;
                }
                // self.cells[i_y][i_x] = Cell::Protected;
                // let index = Self::saturating_cell_index(i_x, i_y);
                // self.cells[index] = Cell::Protected;

                i_x += 1;
            }
            i_y += 1;
        }

        let mut i = 0;
        while i < ship.length {
            if self.is_inside(x, y) {
                self.cells[y][x] = Cell::Ship;
            }
            // self.cells[y][x] = Cell::Ship;
            // let index = Self::saturating_cell_index(x, y);
            // self.cells[index] = Cell::Ship;

            match direction {
                Direction::Horizontal => x += 1,
                Direction::Vetrical => y += 1,
            }

            i += 1;
        }
    }
    // #[inline(never)]
    pub fn place_ship(
        &mut self,
        x: usize,
        y: usize,
        direction: Direction,
        ship: Ship,
    ) -> WasShipplacmentSuccsessfull {
        match direction {
            Direction::Horizontal => {
                debug_assert!(x <= SIZE - ship.length);
                debug_assert!(y < SIZE);
            }
            Direction::Vetrical => {
                debug_assert!(x < SIZE);
                debug_assert!(y <= SIZE - ship.length);
            }
        }
        let placed_ship_board = unsafe {
            PLACED_SHIPS
                .get_unchecked(ship.index)
                .get_unchecked(direction as usize)
                .get_unchecked(y)
                .get_unchecked(x)
        };

        let mut is_placement_allowed = true;
        for (cell, placed_ship_cell) in zip(
            self.cells.as_flattened_mut(),
            placed_ship_board.cells.as_flattened(),
        ) {
            let cell_protects = *cell == Cell::Protected || *cell == Cell::Ship;
            if cell_protects && *placed_ship_cell == Cell::Ship {
                is_placement_allowed = false;
            }
        }

        if !is_placement_allowed {
            return WasShipplacmentSuccsessfull::No;
        }

        for (cell, placed_ship_cell) in zip(
            self.cells.as_flattened_mut(),
            placed_ship_board.cells.as_flattened(),
        ) {
            *cell = (*cell).max(*placed_ship_cell);
        }
        WasShipplacmentSuccsessfull::Yes
    }
    pub const fn cell_index(x: usize, y: usize) -> usize {
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
        me.random_place_ship(*ship_to_place, rng);
        me.place_all_ships_random_recursive(remaining_ships, ship_counts, rng);

        self.random_place_ship(*ship_to_place, rng);
        self.place_all_ships_random_recursive(remaining_ships, ship_counts, rng)
    }
    // #[inline(never)]
    pub fn try_random_place_ship(
        &mut self,
        ship: Ship,
        rng: &mut fastrand::Rng,
    ) -> WasShipplacmentSuccsessfull {
        let lower = (SIZE - ship.length + 1) as u8;
        let higher = SIZE as u8;
        let total = lower * higher * 2;

        let val = rng.u8(0..=total - 1);
        let dir = val % 2 == 0;
        let xy = val / 2;

        let mut x = xy / higher;
        let mut y = xy % higher;

        let dir = match dir {
            true => Direction::Horizontal,
            false => {
                std::mem::swap(&mut x, &mut y);
                Direction::Vetrical
            }
        };

        self.place_ship(x as usize, y as usize, dir, ship)
    }
    #[inline(never)]
    pub fn random_place_ship(&mut self, ship: Ship, rng: &mut fastrand::Rng) {
        while let WasShipplacmentSuccsessfull::No = self.try_random_place_ship(ship, rng) {}
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        // for row in self.cells.iter() {
        //     for cell in row.iter() {
        for row in self.cells.iter().take(SIZE) {
            for cell in row.iter().take(SIZE) {
                f.write_fmt(format_args!("{}", cell))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
