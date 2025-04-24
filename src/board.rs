use crate::ship::Ship;
use crate::{BOARD_SIZE, SIZE};
use glam::{IVec2, ivec2};

use core::iter::Iterator;
use core::simd::u64x4;
use std::fmt::Display;
use std::fmt::Write;
use std::ops::{Index, Sub};
use std::simd::u64x2;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal = 0,
    Vertical = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cell {
    Water = 0,
    Protected = 1,
    ShipHit = 2,
    Ship = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct CellCount {
    counts: [u64; 4],
}
impl CellCount {
    pub fn new() -> Self {
        Self { counts: [0; 4] }
    }
    pub fn add_cell(&mut self, cell: Cell) {
        self.counts[cell as usize] += 1;
    }
}

impl Index<Cell> for CellCount {
    type Output = u64;

    fn index(&self, index: Cell) -> &Self::Output {
        &self.counts[index as usize]
    }
}
impl Sub<CellCount> for CellCount {
    type Output = CellCount;

    fn sub(self, rhs: CellCount) -> Self::Output {
        CellCount {
            counts: [
                self.counts[0] - rhs.counts[0],
                self.counts[1] - rhs.counts[1],
                self.counts[2] - rhs.counts[2],
                self.counts[3] - rhs.counts[3],
            ],
        }
    }
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

    pub fn to_protected(mut self) -> Self {
        for cell in &mut self.cells {
            *cell = match cell {
                Cell::ShipHit | Cell::Ship | Cell::Protected => Cell::Protected,
                Cell::Water => Cell::Water,
            };
        }
        self
    }
    pub fn shifted_protected<const S: Ship>(&self) -> u64x4 {
        let x_shifted = self.multishift(0..S.length()).to_u64x2(Cell::Protected);
        let y_shifted = self
            .multishift((0..S.length()).map(|i| i * SIZE))
            .to_u64x2(Cell::Protected);

        let mut x_mask = Board::new();
        let mut y_mask = Board::new();

        // The last rows the ship can't fit without going over the boarder.
        for y in 0..SIZE {
            for x in 0..SIZE - (S.length() - 1) {
                x_mask.cells[Board::cell_index(x, y)] = Cell::Protected;
                y_mask.cells[Board::cell_index(y, x)] = Cell::Protected;
            }
        }
        let x = !x_shifted & x_mask.to_u64x2(Cell::Protected);
        let y = !y_shifted & y_mask.to_u64x2(Cell::Protected);

        u64x4::from_slice([x.to_array(), y.to_array()].as_flattened())
    }
    fn multishift(&self, amounts: impl Iterator<Item = usize>) -> Self {
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

    pub fn to_u64x2(self, cell_type_to_one: Cell) -> u64x2 {
        let mut val = 0u128;

        for i in 0..u128::BITS as usize {
            let bit_index = i;

            if cell_type_to_one == self.cells[i] {
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
            Direction::Vertical => {
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
                Direction::Vertical => y += 1,
            }

            i += 1;
        }
    }
    pub const fn cell_index(x: usize, y: usize) -> usize {
        x + y * SIZE
    }

    pub fn try_place_ship(mut self, ship: &Ship, pos: IVec2, direction: Direction) -> Option<Self> {
        let top_left = pos;

        let bottom_right = pos
            + match direction {
                Direction::Horizontal => ivec2(ship.length() as i32 - 1, 0),
                Direction::Vertical => ivec2(0, ship.length() as i32 - 1),
            };

        let protected_top_left =
            (top_left - IVec2::ONE).clamp(IVec2::ZERO, IVec2::splat(SIZE as i32 - 1));
        let protected_bottom_right =
            (bottom_right + IVec2::ONE).clamp(IVec2::ZERO, IVec2::splat(SIZE as i32 - 1));

        let mut cell_count_protected = CellCount::new();
        self.map_rect_cells(protected_top_left, protected_bottom_right, |cell| {
            cell_count_protected.add_cell(*cell);
        });

        let mut cell_count_ship = CellCount::new();
        self.map_rect_cells(top_left, bottom_right, |cell| {
            cell_count_ship.add_cell(*cell);
        });

        let only_protected = cell_count_protected - cell_count_ship;

        let is_allowed = cell_count_ship[Cell::Protected] == 0         // would be placed on protected cells
            && cell_count_ship[Cell::Ship] == 0                        // would be placed on ship cells
            && cell_count_ship[Cell::ShipHit] != ship.length() as u64  // would only have hits those are not usefull
            && only_protected[Cell::ShipHit] == 0; // would not use all ship hits

        if !is_allowed {
            return None;
        }

        self.map_rect_cells(top_left, bottom_right, |cell| *cell = Cell::Ship);

        Some(self)
    }
    fn map_rect_cells(
        &mut self,
        top_left: IVec2,
        bottom_right: IVec2,
        mut f: impl FnMut(&mut Cell),
    ) {
        for y in top_left.y..bottom_right.y {
            for x in top_left.x..bottom_right.x {
                f(&mut self.cells[Self::cell_index(x as usize, y as usize)])
            }
        }
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
