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
use std::hash::Hash;
use std::mem::transmute;
use std::ops::{Index, IndexMut};
use std::simd::u64x2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal = 0,
    Vertical = 1,
}
impl Direction {
    const ALL: [Direction; 2] = [Direction::Horizontal, Direction::Vertical];
    pub fn to_ivec2(self) -> IVec2 {
        match self {
            Direction::Horizontal => ivec2(1, 0),
            Direction::Vertical => ivec2(0, 1),
        }
    }
}

enum PlacementResult {
    NotAllowed,
    NoShipHitCovering(ShipPlacement),
    PartialShipHitCovering(ShipPlacement),
    FullShipHitCovering(ShipPlacement),
}
#[derive(Clone)]
pub struct ShipPlacement {
    pub ship: Ship,
    pub pos: IVec2,
    pub board: Board,
    pub direction: Direction,
}
impl ShipPlacement {
    pub fn contains_shot(&self, shot_pos: IVec2) -> bool {
        let lower_bound = self.pos;
        let upper_bound =
            self.pos + IVec2::splat(self.ship.length() as i32 - 1) * self.direction.to_ivec2(); // - 1, because upper bound has to be inclusive
        let contains_shot = shot_pos.x >= lower_bound.x
            && shot_pos.x <= upper_bound.x
            && shot_pos.y >= lower_bound.y
            && shot_pos.y <= upper_bound.y;

        assert_eq!(self.board[shot_pos] == Cell::Ship, contains_shot,);
        contains_shot
    }
}
pub struct AllShipPlacment {
    pub partial_ship_hit_covering: Vec<ShipPlacement>,
    pub full_ship_hit_covering: Vec<ShipPlacement>,
}
impl AllShipPlacment {
    fn add_placement_result(&mut self, placement_result: PlacementResult) {
        match placement_result {
            PlacementResult::NotAllowed => {}
            PlacementResult::NoShipHitCovering(_) => {}
            PlacementResult::PartialShipHitCovering(ship_place) => {
                self.partial_ship_hit_covering.push(ship_place)
            }
            PlacementResult::FullShipHitCovering(ship_place) => {
                self.full_ship_hit_covering.push(ship_place)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
                Cell::Ship | Cell::Protected => Cell::Protected,
                Cell::ShipHit => panic!("not covered ship hit in bitboard: {self}"),
                Cell::Water => Cell::Water,
            };
        }
        this
    }
    pub fn shift_protected<const S: Ship>(&self) -> u64x4 {
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
            x_mask[pos] = Cell::Protected;
            y_mask[pos.yx()] = Cell::Protected;
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

        for i in 0..BOARD_SIZE {
            let bit_index = i;

            if cell_type_to_one == self.cells[i] {
                val |= 1 << bit_index;
            }
        }
        u64x2::from_array([val as u64, (val >> 64) as u64])
    }

    pub fn place_ship(&mut self, x: usize, y: usize, direction: Direction, ship: Ship) {
        let offset = match direction {
            Direction::Horizontal => ivec2(ship.length() as i32, 1),
            Direction::Vertical => ivec2(1, ship.length() as i32),
        };
        let pos = ivec2(x as i32, y as i32);
        self.for_each_rect_cells_mut(pos - IVec2::ONE, pos + offset, |cell| {
            *cell = Cell::Protected
        });

        let ship_offset = direction.to_ivec2() * ship.length() as i32;
        self.for_each_rect_cells_mut(pos, pos + ship_offset, |cell| *cell = Cell::Ship);
    }
    pub fn all_ship_placements_hit_position(
        &self,
        ship_counts: &ShipCounts,
        hit_position: IVec2,
    ) -> AllShipPlacment {
        let mut all_ship_placement = AllShipPlacment {
            partial_ship_hit_covering: Vec::new(),
            full_ship_hit_covering: Vec::new(),
        };
        for ship in ship_counts.iter_ships() {
            for offset in 0..ship.length() {
                let pos = hit_position - Direction::Horizontal.to_ivec2() * offset as i32;
                if pos.x >= 0 && pos.x as usize + ship.length() <= SIZE {
                    let placement_result = self.try_place_ship(ship, pos, Direction::Horizontal);
                    all_ship_placement.add_placement_result(placement_result);
                }
                let pos = hit_position - Direction::Vertical.to_ivec2() * offset as i32;
                if pos.y >= 0 && pos.y as usize + ship.length() <= SIZE {
                    let placement_result = self.try_place_ship(ship, pos, Direction::Vertical);
                    all_ship_placement.add_placement_result(placement_result);
                }
            }
        }
        all_ship_placement
    }
    fn try_place_ship(&self, ship: Ship, pos: IVec2, direction: Direction) -> PlacementResult {
        let top_left = pos;

        let bottom_right = pos + direction.to_ivec2() * (ship.length() - 1) as i32;

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

        let is_allowed = cell_count_ship[Cell::Water] + cell_count_ship[Cell::ShipHit] == ship.length() as u64
            // only allow to be placed on water or ship hits
            && only_protected[Cell::ShipHit] == 0; // would not use all ship hits

        if !is_allowed {
            return PlacementResult::NotAllowed;
        }

        let mut this = self.clone();
        this.for_each_rect_cells_mut(protected_top_left, protected_bottom_right, |cell| {
            *cell = Cell::Protected
        });
        this.for_each_rect_cells_mut(top_left, bottom_right, |cell| *cell = Cell::Ship);

        let ship_placement = ShipPlacement {
            ship,
            pos,
            direction,
            board: this,
        };

        if cell_count_ship[Cell::ShipHit] == 0 {
            PlacementResult::NoShipHitCovering(ship_placement)
        } else if (cell_count_ship[Cell::ShipHit] as usize) < ship.length() {
            PlacementResult::PartialShipHitCovering(ship_placement)
        } else {
            PlacementResult::FullShipHitCovering(ship_placement)
        }
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
            f(&mut self[pos])
        }
    }
    /// Maps a function over a rectangular area of the board. Bottom right is inclusive.
    /// Clamps the cords to the size of the board.
    fn for_each_rect_cells(&self, top_left: IVec2, bottom_right: IVec2, mut f: impl FnMut(&Cell)) {
        for pos in rect_iter(top_left, bottom_right + IVec2::ONE) {
            f(&self[pos])
        }
    }
    pub fn to_bitboard<const N: usize>(&self, ship_counts: ShipCounts) -> BitBoard<N> {
        BitBoard::new(self, ship_counts)
    }

    pub fn has_ship_hits(&self) -> bool {
        self.cells.contains(&Cell::ShipHit)
    }
}
impl Index<IVec2> for Board {
    type Output = Cell;

    fn index(&self, index: IVec2) -> &Self::Output {
        &self.cells[index.x as usize + index.y as usize * SIZE]
    }
}
impl IndexMut<IVec2> for Board {
    fn index_mut(&mut self, index: IVec2) -> &mut Self::Output {
        &mut self.cells[index.x as usize + index.y as usize * SIZE]
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

impl Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let own_bytes: [u8; BOARD_SIZE] = unsafe { transmute(self.clone()) };
        state.write(&own_bytes[0..SIZE * SIZE]);
    }
}
