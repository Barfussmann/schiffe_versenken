use crate::bit_board::BitBoard;
use crate::cell::{Cell, CellCount};
use crate::ship::{Ship, ShipCounts};
use crate::utils::rect_iter;
use crate::{BOARD_SIZE, SIZE};
use glam::{IVec2, Vec2Swizzles, ivec2};

use core::iter::Iterator;
use core::simd::u64x4;
use std::fmt::Display;
use std::fmt::Write;
use std::simd::u64x2;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal = 0,
    Vertical = 1,
}

const MAX_POS: IVec2 = IVec2::splat(SIZE as i32 - 1);

#[derive(Debug, Clone)]
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

    pub fn to_protected(&self) -> Self {
        let mut this = self.clone();
        for cell in &mut this.cells {
            *cell = match cell {
                // Cell::Ship => Cell::Protected,
                // _ => Cell::Water,
                Cell::ShipHit | Cell::Ship | Cell::Protected => Cell::Protected,
                Cell::Water => Cell::Water,
            };
        }
        this
    }
    pub fn shifted_protected<const S: Ship>(&self) -> u64x4 {
        let x_shifted = self.multishift(0..S.length()).to_u64x2(Cell::Protected);
        let y_shifted = self
            .multishift((0..S.length()).map(|i| i * SIZE))
            .to_u64x2(Cell::Protected);

        let mut x_mask = Board::new();
        let mut y_mask = Board::new();

        // The last rows the ship can't fit without going over the boarder.
        for pos in rect_iter(
            IVec2::ZERO,
            ivec2((SIZE - (S.length() - 1)) as i32, SIZE as i32),
        ) {
            x_mask.cells[Board::cell_index(pos)] = Cell::Protected;
            y_mask.cells[Board::cell_index(pos.yx())] = Cell::Protected;
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

    pub fn to_u64x2(&self, cell_type_to_one: Cell) -> u64x2 {
        let mut val = 0u128;

        for i in 0..u128::BITS as usize {
            let bit_index = i;

            if cell_type_to_one == self.cells[i] {
                val |= 1 << bit_index;
            }
        }
        u64x2::from_array([val as u64, (val >> 64) as u64])
    }

    pub fn const_place_ship(&mut self, x: usize, y: usize, direction: Direction, ship: Ship) {
        let offset = match direction {
            Direction::Horizontal => ivec2(ship.length() as i32, 1),
            Direction::Vertical => ivec2(1, ship.length() as i32),
        };
        let pos = ivec2(x as i32, y as i32);
        self.for_each_rect_cells_mut(pos - IVec2::ONE, pos + offset, |cell| {
            *cell = Cell::Protected
        });

        let ship_offset = match direction {
            Direction::Horizontal => ivec2(ship.length() as i32, 0),
            Direction::Vertical => ivec2(0, ship.length() as i32),
        };
        self.for_each_rect_cells_mut(pos, pos + ship_offset, |cell| *cell = Cell::Ship);
    }
    pub const fn cell_index(pos: IVec2) -> usize {
        pos.x as usize + pos.y as usize * SIZE
    }
    pub fn all_ship_placements(
        &self,
        ship_counts: &ShipCounts,
    ) -> impl Iterator<Item = (Board, Ship)> {
        std::iter::from_coroutine(
            #[coroutine]
            move || {
                for ship in ship_counts.iter_ships().collect::<Vec<_>>() {
                    for pos in rect_iter(IVec2::ZERO, IVec2::splat(SIZE as i32)) {
                        if pos.x as usize + ship.length() <= SIZE {
                            yield self
                                .try_place_ship(ship, pos, Direction::Horizontal)
                                .map(|board| (board, ship))
                        }
                        if pos.y as usize + ship.length() <= SIZE {
                            yield self
                                .try_place_ship(ship, pos, Direction::Vertical)
                                .map(|board| (board, ship))
                        }
                    }
                }
            },
        )
        .flatten()
    }
    pub fn try_place_ship(&self, ship: Ship, pos: IVec2, direction: Direction) -> Option<Self> {
        let top_left = pos;

        let bottom_right = pos
            + match direction {
                Direction::Horizontal => ivec2(ship.length() as i32 - 1, 0),
                Direction::Vertical => ivec2(0, ship.length() as i32 - 1),
            };

        let protected_top_left = top_left - IVec2::ONE;
        let protected_bottom_right = bottom_right + IVec2::ONE;

        let mut cell_count_protected = CellCount::new();
        self.for_each_rect_cells(protected_top_left, protected_bottom_right, |cell| {
            cell_count_protected.add_cell(*cell);
        });

        let mut cell_count_ship = CellCount::new();
        self.for_each_rect_cells(top_left, bottom_right, |cell| {
            cell_count_ship.add_cell(*cell);
        });

        let only_protected = cell_count_protected - cell_count_ship;

        let allowable_hit_range = 1..ship.length() as u64; // Needs atleast one hit to be placed and if it's only hits it is'nt usefull

        let is_allowed = cell_count_ship[Cell::Protected] == 0         // would be placed on protected cells
            && cell_count_ship[Cell::Ship] == 0                        // would be placed on ship cells
            && allowable_hit_range.contains(&cell_count_ship[Cell::ShipHit])
            && only_protected[Cell::ShipHit] == 0; // would not use all ship hits

        if !is_allowed {
            return None;
        }

        let mut this = self.clone();
        this.for_each_rect_cells_mut(protected_top_left, protected_bottom_right, |cell| {
            *cell = Cell::Protected
        });
        this.for_each_rect_cells_mut(top_left, bottom_right, |cell| *cell = Cell::Ship);

        Some(this)
    }
    /// Maps a function over a rectangular area of the board. Bottom right is inclusive.
    /// Clamps the cords to the size of the board.
    fn for_each_rect_cells_mut(
        &mut self,
        top_left: IVec2,
        bottom_right: IVec2,
        mut f: impl FnMut(&mut Cell),
    ) {
        for pos in rect_iter(top_left, bottom_right + IVec2::ONE) {
            f(&mut self.cells[Self::cell_index(pos)])
        }
    }
    /// Maps a function over a rectangular area of the board. Bottom right is inclusive.
    /// Clamps the cords to the size of the board.
    fn for_each_rect_cells(&self, top_left: IVec2, bottom_right: IVec2, mut f: impl FnMut(&Cell)) {
        for pos in rect_iter(top_left, bottom_right + IVec2::ONE) {
            f(&self.cells[Self::cell_index(pos)])
        }
    }
    pub fn to_bitboard<const N: usize>(&self, ship_counts: ShipCounts) -> BitBoard<N> {
        BitBoard::new(self, ship_counts)
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
