use crate::Ship;
use crate::{BOARD_SIZE, SHIPS, SIZE};

static PLACED_BITS_SHIPS: [[[[BitBoard; SIZE]; SIZE]; 2]; 10] = {
    let mut placed_ships = [[[[BitBoard::new(Board::new()); SIZE]; SIZE]; 2]; 10];

    let mut ship_i = 0;
    while ship_i < 10 {
        let mut dir_i = 0;
        while dir_i < 2 {
            let mut y = 0;
            while y < SIZE {
                let mut x = 0;
                while x < SIZE {
                    placed_ships[ship_i][dir_i][y][x] =
                        BitBoard::new(PLACED_SHIPS[ship_i][dir_i][y][x]);

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

static PLACED_SHIPS: [[[[Board; SIZE]; SIZE]; 2]; 10] = {
    let mut placed_ships = [[[[Board::new(); SIZE]; SIZE]; 2]; 10];

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

                    placed_ships[ship_index][dir_i][y][x].const_place_ship(x, y, dir, ship);

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
use std::mem::transmute;
use std::simd::u64x2;

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

#[derive(Clone, Copy)]
#[repr(align(32))]
pub struct BitBoard {
    pub protected: u64x2,
    pub ship: u64x2,
}
impl BitBoard {
    pub const fn new(board: Board) -> Self {
        let mut protected = 0u128;
        let mut ship = 0u128;

        let mut i = 0;

        while i < u128::BITS as usize {
            let bit_index = Self::map_index_to_bit_index(i);
            match board.cells[i] {
                Cell::Water => {}
                Cell::Protected => protected |= 1 << bit_index,
                Cell::Ship | Cell::ShipHit => {
                    ship |= 1 << bit_index;
                    protected |= 1 << bit_index
                }
            }

            i += 1;
        }
        Self {
            protected: unsafe { transmute::<u128, u64x2>(protected) },
            ship: unsafe { transmute::<u128, u64x2>(ship) },
        }
    }
    const fn map_index_to_bit_index(index: usize) -> usize {
        if index < 40 {
            index
        } else {
            (index - 40) + 64 // put it in to the next u64 to make further calculations easier
        }
    }
    // #[inline(never)]
    fn place_ship(&mut self, index: usize, ship: Ship) {
        let placed_ship_board = unsafe {
            PLACED_BITS_SHIPS
                .get_unchecked(ship.index)
                .as_flattened()
                .as_flattened()
                .get_unchecked(index)
        };

        self.protected |= placed_ship_board.protected;
        self.ship |= placed_ship_board.ship;
    }
    // #[inline(never)]
    fn allowable_ship_placements(&self, ship: Ship) -> (u64x2, u64x2) {
        const Y_SHIP_MASK: [u64x2; 8] = {
            let mut masks = [u64x2::from_array([0; 2]); 8];

            let mut ship_length = 1;
            while ship_length < masks.len() {
                let low = (1 << (4 * SIZE)) - 1; // group of the lower 4 rows
                let high = (1 << ((7 - ship_length) * SIZE)) - 1; // when the ship length is over 1 the top rows are cut off

                masks[ship_length] = u64x2::from_array([low, high]);
                ship_length += 1;
            }

            masks
        };
        const X_SHIP_MASK: [u64x2; 8] = {
            let mut masks = [u64x2::from_array([0; 2]); 8];

            let mut ship_length = 1;
            while ship_length < masks.len() {
                let mut mask = 0u128;

                let single_row_allowable = (1 << (SIZE - (ship_length - 1))) - 1;

                let mut y = 0;
                while y < SIZE {
                    let bit_index = BitBoard::map_index_to_bit_index(y * SIZE);

                    mask |= single_row_allowable << bit_index;
                    y += 1
                }

                masks[ship_length] = unsafe { transmute::<u128, u64x2>(mask) };
                ship_length += 1;
            }

            masks
        };

        let allowable = !self.protected;

        let mut ship_placements_x = allowable;
        for i in 1..ship.length() as u64 {
            ship_placements_x &= allowable >> i;
        }
        ship_placements_x &= X_SHIP_MASK[ship.length()];

        let mut ship_placements_y = allowable;
        for i in 1..ship.length() {
            let high = allowable[1];
            let high_shifted_by_one = high >> SIZE;

            let low_shifted_by_one = (allowable[0] >> SIZE) & ((1 << 40) - 1);
            let low_with_high = low_shifted_by_one | (high << 30);

            ship_placements_y &=
                u64x2::from_array([low_with_high, high_shifted_by_one]) >> ((i - 1) * SIZE) as u64;
        }
        ship_placements_y &= Y_SHIP_MASK[ship.length()];

        (ship_placements_x, ship_placements_y)
    }
    #[inline(never)]
    pub fn random_place_ship(&mut self, ship: Ship, random_value: u32, rng: &mut fastrand::Rng) {
        let (ship_placements_x, ship_placements_y) = self.allowable_ship_placements(ship);
        let x_ships = ship_placements_x[0].count_ones() + ship_placements_x[1].count_ones();
        let y_ships = ship_placements_y[0].count_ones() + ship_placements_y[1].count_ones();

        let total_index = random_value % (x_ships + y_ships);
        // let total_index = rng.u8(..(x_ships + y_ships) as u8) as u32;

        let (bit_map, index, offset) = if total_index < x_ships {
            (ship_placements_x, total_index, 0)
        } else {
            (ship_placements_y, total_index - x_ships, SIZE * SIZE) // there is a brache for ship_placments because ther are 128 and not 64 but and thus can't be move by cmov
        };
        let index = set_bit_to_index(bit_map, index) as usize + offset;

        // let index = if total_index < x_ships {
        //     set_bit_to_index(ship_placements_x, total_index) as usize
        // } else {
        //     // 0
        //     (SIZE * SIZE) + set_bit_to_index(ship_placements_y, total_index - x_ships) as usize
        // };

        self.place_ship(index, ship)
    }
}

fn nth_set_bit_index_u64(num: u64, n: u32) -> u32 {
    let spread_bits = unsafe { std::arch::x86_64::_pdep_u64(1 << n, num) };
    spread_bits.trailing_zeros()
}
// #[inline(never)]
fn set_bit_to_index(set_bits: u64x2, valid_ship_index: u32) -> u32 {
    // num[0].count_ones() + num[1].count_ones()

    let low_set_bits = set_bits[0].count_ones();

    let (target_n, num, offset) = if valid_ship_index < low_set_bits {
        (valid_ship_index, set_bits[0], 0)
    } else {
        (valid_ship_index - low_set_bits, set_bits[1], 40)
    };
    nth_set_bit_index_u64(num, target_n) + offset

    // 'outer: {
    //     let mut count = 0;
    //     for i in 0..128 {
    //         if num & (1 << i) != 0 {
    //             if count == n {
    //                 break 'outer i;
    //             }
    //             count += 1;
    //         }
    //     }
    //     0
    // };
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
