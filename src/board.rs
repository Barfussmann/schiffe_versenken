use crate::Ship;
use crate::{BOARD_SIZE, SHIPS, SIZE};

static PLACED_BITS_SHIPS: [[[[BitBoard; SIZE]; SIZE]; 2]; 10] = {
    let mut placed_ships = [[[[BitBoard::new(Board::new()); SIZE]; SIZE]; 2]; 10];

    let mut ship_i = 0;
    while ship_i < SHIPS.len() {
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
    pub protected: u128,
    pub ship: u128,
}
impl BitBoard {
    pub const fn new(board: Board) -> Self {
        let mut me = Self {
            protected: 0,
            ship: 0,
        };
        let mut i = 0;

        while i < u128::BITS as usize {
            match board.cells[i] {
                Cell::Water => {}
                Cell::Protected => me.protected |= 1 << i,
                Cell::Ship | Cell::ShipHit => {
                    me.ship |= 1 << i;
                    me.protected |= 1 << i
                }
            }

            i += 1;
        }
        me
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
    fn allowable_ship_placements(&self, ship: Ship) -> (u128, u128) {
        const Y_SHIP_MASK: [u128; 8] = {
            let mut masks = [0; 8];

            let mut ship_length = 1;
            while ship_length < masks.len() {
                masks[ship_length] = ((1 << (SIZE * SIZE)) - 1) >> (SIZE * (ship_length - 1));
                ship_length += 1;
            }

            masks
        };
        const X_SHIP_MASK: [u128; 8] = {
            let mut masks = [0; 8];

            let mut ship_length = 1;
            while ship_length < masks.len() {
                let mut mask = 0;

                let single_row_allowable = (1 << (SIZE - (ship_length - 1))) - 1;

                let mut y = 0;
                while y < SIZE {
                    mask |= single_row_allowable << (y * SIZE);
                    y += 1
                }

                masks[ship_length] = mask;
                ship_length += 1;
            }

            masks
        };

        let allowable = !self.protected;

        let mut ship_placements_x = allowable;
        for i in 1..ship.length() {
            ship_placements_x &= allowable >> i;
        }
        ship_placements_x &= X_SHIP_MASK[ship.length()];

        let mut ship_placements_y = allowable;
        for i in 1..ship.length() {
            ship_placements_y &= allowable >> (i * SIZE);
        }
        ship_placements_y &= Y_SHIP_MASK[ship.length()];

        (ship_placements_x, ship_placements_y)
    }
    // #[inline(never)]
    pub fn random_place_ship(&mut self, ship: Ship, rng: &mut fastrand::Rng) {
        let (ship_placements_x, ship_placements_y) = self.allowable_ship_placements(ship);
        let x_ships = ship_placements_x.count_ones();
        let y_ships = ship_placements_y.count_ones();

        let total_index = rng.u8(..(x_ships + y_ships) as u8) as usize;

        let index = if total_index < x_ships as usize {
            nth_set_bit_index(ship_placements_x, total_index as u32) as usize
        } else {
            (SIZE * SIZE)
                + nth_set_bit_index(ship_placements_y, (total_index - x_ships as usize) as u32)
                    as usize
        };

        self.place_ship(index, ship)
    }
}

fn nth_set_bit_index_u64(num: u64, n: u32) -> u32 {
    let spread_bits = unsafe { std::arch::x86_64::_pdep_u64(1 << n, num) };
    spread_bits.trailing_zeros()
}
// #[inline(never)]
fn nth_set_bit_index(num: u128, n: u32) -> u32 {
    // let low = num as u64;
    // let high = (num >> 64) as u64;
    // let low_set_bits = low.count_ones();

    // if n < low_set_bits {
    //     nth_set_bit_index_u64(low, n)
    // } else {
    //     64 + nth_set_bit_index_u64(high, n - low_set_bits)
    // }

    let low = num as u64;
    let high = (num >> 64) as u64;
    let low_set_bits = low.count_ones();

    let (target_n, num, offset) = if n < low_set_bits {
        (n, low, 0)
    } else {
        (n - low_set_bits, high, 64)
    };
    nth_set_bit_index_u64(num, target_n) + offset

    // let right_result = 'outer: {
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
    // println!("wrong: {:2}, right: {:2}", result, right_result);
    // right_result
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
