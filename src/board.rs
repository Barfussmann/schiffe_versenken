use crate::ship::Ship;
use crate::{BOARD_SIZE, SHIPS, SIZE};

use std::fmt::Display;
use std::fmt::Write;

pub static PLACED_SHIPS: [[Board; 256]; 10] = {
    let mut placed_ships = [[Board::new(); 256]; 10];

    let mut ship_i = 0;
    while ship_i < SHIPS.len() {
        let ship = SHIPS[ship_i];
        let ship_index = ship.index;
        let mut dir_i = 0;
        while dir_i < 2 {
            let mut y = 0;
            while y < SIZE {
                let mut x = 0;
                while x < SIZE {
                    let dir = match dir_i {
                        0 => Direction::Horizontal,
                        1 => Direction::Vetrical,
                        _ => unreachable!(),
                    };

                    let index = dir_i * 128 + y * 10 + x;
                    placed_ships[ship_index][index].const_place_ship(x, y, dir, ship);

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

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Cell::Protected => " o ",
            Cell::Water => " _ ",
            Cell::ShipHit | Cell::Ship => " X ",
        };
        f.write_str(str)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(align(128))]
pub struct Board {
    pub cells: [Cell; BOARD_SIZE],
}

impl Board {
    pub const fn new() -> Board {
        Board {
            cells: [Cell::Water; BOARD_SIZE],
        }
    }
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
                width = ship.length() + 2;
                height = 3;
            }
            Direction::Vetrical => {
                width = 3;
                height = ship.length() + 2;
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
                let index = Self::saturating_cell_index(i_x, i_y);
                self.cells[index] = Cell::Protected;

                i_x += 1;
            }
            i_y += 1;
        }

        let mut i = 0;
        while i < ship.length() {
            let index = Self::saturating_cell_index(x, y);
            self.cells[index] = Cell::Ship;

            match direction {
                Direction::Horizontal => x += 1,
                Direction::Vetrical => y += 1,
            }

            i += 1;
        }
    }
    pub const fn cell_index(x: usize, y: usize) -> usize {
        x + y * SIZE
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

pub fn set_protecet_at_offsets(
    x: usize,
    y: usize,
    start_board: &mut Board,
    offsets: [(i32, i32); 4],
) {
    for corner_offset in offsets {
        let corner_x = x as i32 + corner_offset.0;
        let corner_y = y as i32 + corner_offset.1;
        let range = 0..SIZE as i32;
        if !range.contains(&corner_x) | !range.contains(&corner_y) {
            continue;
        }
        let cell = &mut start_board.cells[Board::cell_index(corner_x as usize, corner_y as usize)];
        match cell {
            Cell::Water => *cell = Cell::Protected,
            Cell::Protected | Cell::Ship | Cell::ShipHit => {}
        }
    }
}
