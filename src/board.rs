use crate::ship::Ship;
use crate::{BOARD_SIZE, SHIPS, SIZE};

use std::fmt::Display;
use std::fmt::Write;
use std::simd::u64x2;
use std::sync::LazyLock;

pub static PLACED_SHIPS: LazyLock<Box<[[Board; 256]; 4]>> = LazyLock::new(|| {
    let mut placed_ships = [[Board::new(); 256]; 4];
    for ship in SHIPS {
        for dir in [Direction::Horizontal, Direction::Vetrical] {
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let index = dir as usize * 128 + y * 10 + x;
                    placed_ships[ship.index()][index].const_place_ship(x, y, dir, *ship);
                }
            }
        }
    }
    Box::new(placed_ships)
});

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
    pub const fn map_index_to_bit_index(index: usize) -> usize {
        if index < 40 {
            index
        } else {
            (index - 40) + 64 // put it into the next u64 to make further calculations easier
        }
    }
    pub fn to_protected(mut self) -> Self {
        for cell in &mut self.cells {
            *cell = match cell {
                Cell::Protected => Cell::Protected,
                Cell::ShipHit | Cell::Ship => Cell::Protected,
                Cell::Water => Cell::Water,
            };
        }
        self
    }
    pub fn shifted_protected<const S: Ship>(&self) -> (u64x2, u64x2) {
        let shifts_x: Vec<usize> = (0..S.length()).collect();
        let shifts_y: Vec<usize> = (0..S.length() * SIZE).step_by(SIZE).collect();

        let x_ship_mask = {
            let mut mask = 0u128;
            let single_row_allowable = (1 << (SIZE - (S.length() - 1))) - 1;
            for y in 0..SIZE {
                let bit_index = Self::map_index_to_bit_index(y * SIZE);
                mask |= single_row_allowable << bit_index;
            }
            unsafe { std::mem::transmute::<u128, u64x2>(mask) }
        };
        let y_ship_mask = {
            let low = (1 << (4 * SIZE)) - 1; // group of the lower 4 rows
            let high = (1 << ((7 - S.length()) * SIZE)) - 1; // when the ship length is over 1 the top rows are cut off
            u64x2::from_array([low, high])
        };

        (
            !self.multishift(&shifts_x).to_u64x2(Cell::Protected) & x_ship_mask,
            !self.multishift(&shifts_y).to_u64x2(Cell::Protected) & y_ship_mask,
        )
    }
    fn multishift(&self, amounts: &[usize]) -> Self {
        let mut result = Self::new();
        for shift in amounts {
            for i in 0..SIZE * SIZE {
                let shifted = self.cells.get(i + shift).unwrap_or(&Cell::Water);

                if *shifted == Cell::Protected {
                    result.cells[i] = Cell::Protected;
                }
            }
        }
        result
    }

    pub fn to_u64x2(self, cell_type: Cell) -> u64x2 {
        let mut val = 0u128;

        for i in 0..u128::BITS as usize {
            let bit_index = Self::map_index_to_bit_index(i);

            if cell_type == self.cells[i] {
                val |= 1 << bit_index;
            }
        }
        u64x2::from_array([val as u64, (val >> 64) as u64])
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
        }
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

    pub fn swab(mut self, a: Cell, b: Cell) -> Self {
        for cell in self.cells.iter_mut() {
            if *cell == a {
                *cell = b;
            } else if *cell == b {
                *cell = a;
            }
        }
        self
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        for row in self.cells.chunks(SIZE).take(SIZE) {
            for cell in row {
                f.write_fmt(format_args!("{cell}"))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
