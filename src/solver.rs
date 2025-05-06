use std::{
    iter::zip,
    simd::prelude::*,
    sync::LazyLock,
    time::{Duration, Instant},
};

use crate::{
    SIZE,
    bit_board::BitBoard,
    board::{Board, Cell, Direction},
    ship::Ship,
    ship_counts::{self, ShipCounts, ShipPositionCounts, SmallShipPositionCounts},
};
use num_format::{Locale, ToFormattedString};
use rand::random;
use rayon::prelude::*;

pub struct PlacedBitShips {
    pub placed_ships: [[BitBoard; 256]; 6],
}
impl PlacedBitShips {
    pub fn new() -> &'static Self {
        const SHIPS: [Ship; 6] = [
            Ship::new(1, 0),
            Ship::new(2, 0),
            Ship::new(3, 0),
            Ship::new(4, 0),
            Ship::new(5, 0),
            Ship::new(6, 0),
        ];

        static PLACED_BIT_SHIPS: LazyLock<PlacedBitShips> = LazyLock::new(|| {
            assert!(
                SHIPS.is_sorted_by_key(|ship| ship.index()),
                "SHIPS has to be sorted by ship.index()"
            );
            let placed_ships = SHIPS.map(|ship| {
                let mut placed_ships = [BitBoard::new(Board::new()); 256];
                for dir in [Direction::Horizontal, Direction::Vertical] {
                    for y in 0..SIZE {
                        for x in 0..SIZE {
                            let bit_board_index = dir as usize * 128 + (y * 10 + x);
                            let mut board = Board::new();
                            board.const_place_ship(x, y, dir, ship);

                            placed_ships[bit_board_index] = BitBoard::new(board);
                        }
                    }
                }
                placed_ships
            });
            PlacedBitShips { placed_ships }
        });
        &PLACED_BIT_SHIPS
    }
}

pub struct SpecialRng {
    aes_key: u64x8,
    aes_value: u64x8,
}
impl SpecialRng {
    fn new() -> Self {
        Self {
            aes_key: u64x8::from_array(random()),
            aes_value: u64x8::splat(0),
        }
    }
    // #[inline(never)]
    pub fn extra_wide_get_random(&mut self, upper_range: u64x8) -> u64x8 {
        let ret = unsafe {
            std::arch::x86_64::_mm512_mulhi_epu16(self.aes_value.into(), upper_range.into())
        }
        .into();
        self.step_aes();
        ret
    }
    pub fn wide_get_random(&mut self, upper_range: u64x4) -> u64x4 {
        let ret = unsafe {
            std::arch::x86_64::_mm256_mulhi_epu16(
                simd_swizzle!(self.aes_value, [0, 1, 2, 3]).into(),
                upper_range.into(),
            )
        }
        .into();
        self.step_aes();
        ret
    }
    pub fn get_random_u16(&mut self, upper_range: u16x8) -> u16x8 {
        let ret = unsafe {
            std::arch::x86_64::_mm_mulhi_epu16(
                simd_swizzle!(self.aes_value, [0, 1]).into(),
                upper_range.into(),
            )
        }
        .into();
        self.step_aes();
        ret
    }
    pub fn extra_wide_get_random_u16_(&mut self, upper_range: u16x32) -> u16x32 {
        let ret = unsafe {
            std::arch::x86_64::_mm512_mulhi_epu16(self.aes_value.into(), upper_range.into())
        }
        .into();
        self.step_aes();
        ret
    }
    fn step_aes(&mut self) {
        unsafe {
            self.aes_value =
                std::arch::x86_64::_mm512_aesenc_epi128(self.aes_value.into(), self.aes_key.into())
                    .into();
        }
    }
}

pub struct Solver {
    pub placed_bit_ships: &'static PlacedBitShips,
    pub current_board: Board,
}

impl Solver {
    pub fn new() -> Self {
        Solver {
            placed_bit_ships: PlacedBitShips::new(),
            current_board: Board::new(),
        }
    }

    pub fn reset(&mut self) {
        self.current_board = Board::new();
    }
    pub fn run(&self, time_to_run: Duration, ship_amounts: [u8; 6]) {
        let start_time = Instant::now();
        let ship_counts = self.inner_loop(time_to_run, ship_amounts);

        let max_index = zip(
            ship_counts.counts.iter().enumerate(),
            &self.current_board.cells,
        )
        .filter(|(_, cell)| **cell == Cell::Water)
        .max_by_key(|((_, count), _)| **count)
        .unwrap()
        .0
        .0;

        let elapsed_time = start_time.elapsed();
        println!(
            "in: {:4.3?} calculated: {:12}",
            elapsed_time,
            ship_counts.board_count.to_formatted_string(&Locale::en)
        );
        println!("ship_counts: {ship_counts}");
        println!(
            "total_ships: {}",
            ship_counts.counts.iter().sum::<u64>() as f64 / ship_counts.board_count as f64
        );

        let x = max_index % SIZE;
        let y = max_index / SIZE;

        println!("Max (x, y): ({}, {})", (x as u8 + b'A') as char, y + 1);
    }
    pub fn inner_loop(
        &self,
        time_to_run: Duration,
        ship_amounts: [u8; 6],
    ) -> ship_counts::ShipCounts {
        // pub fn inner_loop(&mut self, random_values: &[[u32; SHIPS.len()]]) -> ship_counts::ShipCounts {
        let bit_board = BitBoard::new(self.current_board);

        let start_time = Instant::now();
        (0..rayon::current_num_threads())
            .par_bridge()
            .map(|_| {
                let mut ship_counts = ship_counts::ShipCounts::new();
                let mut special_rng = SpecialRng::new();

                let end_time = start_time + time_to_run;
                while Instant::now() < end_time {
                    step_summing::<{ Ship::new(5, 4) }>(
                        bit_board,
                        &mut ship_counts,
                        self.placed_bit_ships,
                    );
                    // step(
                    //     bit_board,
                    //     ship_amounts,
                    //     &mut ship_counts,
                    //     self.placed_bit_ships,
                    //     &mut special_rng,
                    // );
                }
                ship_counts
            })
            .reduce(ship_counts::ShipCounts::new, |mut a, b| {
                a.add_other_count(b);
                a
            })
    }
}

const SHIP_COUNT: usize = 5;
#[rustfmt::skip]
fn step_inner<
    const SHIP: Ship,
    const NEXT_SHIP_INDEX: [usize; SHIP_COUNT],
>(
    board: BitBoard,
    counts: &mut [ShipPositionCounts; 5],
    placed_bit_ships: &PlacedBitShips,
) -> u64 {
    // directly add the the positions of all possible placements of the last ship
    if const { remaining_ships(SHIP.index(), NEXT_SHIP_INDEX) } == 0 {
        return counts[SHIP.index].small_counts.add_possible_ship_positions(board.allowable::<SHIP>())
        // return 1
    }

    let mut configurations = 0;
    for ship_pos in BitIter::new(board.allowable::<SHIP>()) {
        let mut board = board;
        board.place_ship::<SHIP>(ship_pos, placed_bit_ships);

        let additional_configurations = match NEXT_SHIP_INDEX[SHIP.index] {
            0 => step_inner::<{ Ship::new(2, 0) }, NEXT_SHIP_INDEX>(board, counts, placed_bit_ships),
            1 => step_inner::<{ Ship::new(3, 1) }, NEXT_SHIP_INDEX>(board, counts, placed_bit_ships),
            2 => step_inner::<{ Ship::new(3, 2) }, NEXT_SHIP_INDEX>(board, counts, placed_bit_ships),
            3 => step_inner::<{ Ship::new(4, 3) }, NEXT_SHIP_INDEX>(board, counts, placed_bit_ships),
            4 => step_inner::<{ Ship::new(5, 4) }, NEXT_SHIP_INDEX>(board, counts, placed_bit_ships),
            _ => unreachable!()
            // _ => {0}
        };
        // addes the currently placed ship with the amount of differnt configurations
        counts[SHIP.index].add_single_ship(ship_pos, additional_configurations);

        configurations += additional_configurations;
    }
    // flush the small count with the u8 to the big u64 nums to prevent overflow
    if const { remaining_ships(SHIP.index(), NEXT_SHIP_INDEX) } == 1 {
        counts[NEXT_SHIP_INDEX[SHIP.index]].add_small();
    }
    configurations
}
const fn remaining_ships(ship_index: usize, next_ship_index: [usize; SHIP_COUNT]) -> usize {
    let mut ship_count = 0;
    let mut ship_index = ship_index;
    while ship_index < SHIP_COUNT {
        ship_index = next_ship_index[ship_index];
        ship_count += 1;
    }
    ship_count - 1
}

pub fn step(bit_board: BitBoard, ship_counts: &mut ShipCounts, placed_bit_ships: &PlacedBitShips) {}
#[inline(never)]
pub fn step_summing<const STARTING_SHIP: Ship>(
    bit_board: BitBoard,
    ship_counts: &mut ship_counts::ShipCounts,
    placed_bit_ships: &PlacedBitShips,
) {
    let mut counts: [_; 5] = std::array::from_fn(|_| ShipPositionCounts::new());
    let total_boards = step_inner::<STARTING_SHIP, { [255, 0, 1, 2, 3] }>(
        bit_board,
        &mut counts,
        placed_bit_ships,
    );
    ship_counts.board_count += total_boards;
    ship_counts.add_ship_positions_counts::<{ Ship::new(2, 0) }>(&mut counts[0]);
    ship_counts.add_ship_positions_counts::<{ Ship::new(3, 0) }>(&mut counts[1]);
    ship_counts.add_ship_positions_counts::<{ Ship::new(3, 0) }>(&mut counts[2]);
    ship_counts.add_ship_positions_counts::<{ Ship::new(4, 0) }>(&mut counts[3]);
    ship_counts.add_ship_positions_counts::<{ Ship::new(5, 0) }>(&mut counts[4]);
}

#[derive(Clone, Copy)]
pub struct BitIter {
    current_u64: u64,
    bits: u64x4,
    offset: u8,
    //     indecies: [u8; 256],
    //     len: u8,
}
impl BitIter {
    // #[inline(never)]
    pub fn new(bits: u64x4) -> Self {
        // let mut indecies = [0; 256];
        // let nums_0 = u8x64::from_array(std::array::from_fn(|i| i as u8));
        // let nums_1 = u8x64::from_array(std::array::from_fn(|i| i as u8 + 64));
        // let nums_2 = u8x64::from_array(std::array::from_fn(|i| i as u8 + 128));
        // let nums_3 = u8x64::from_array(std::array::from_fn(|i| i as u8 + 192));

        // let offset_1 = bits[0].count_ones();
        // let offset_2 = bits[0].count_ones() + bits[1].count_ones();
        // let offset_3 = bits[0].count_ones() + bits[1].count_ones() + bits[2].count_ones();
        // let len = (bits[0].count_ones()
        //     + bits[1].count_ones()
        //     + bits[2].count_ones()
        //     + bits[3].count_ones()) as u8;

        // #[rustfmt::skip]
        // unsafe {
        //     _mm512_mask_compressstoreu_epi8(indecies.as_mut_ptr(), bits[0], nums_0.into());
        //     _mm512_mask_compressstoreu_epi8(indecies.as_mut_ptr().add(offset_1 as usize), bits[1], nums_1.into());
        //     _mm512_mask_compressstoreu_epi8(indecies.as_mut_ptr().add(offset_2 as usize), bits[2], nums_2.into());
        //     _mm512_mask_compressstoreu_epi8(indecies.as_mut_ptr().add(offset_3 as usize), bits[3], nums_3.into());
        // }

        Self {
            current_u64: bits[0],
            bits,
            offset: 0,
            // indecies,
            // len,
        }
    }
    fn dummy() -> Self {
        Self::new(u64x4::splat(0))
    }
}
impl Iterator for BitIter {
    type Item = u8;

    // #[inline(never)]
    fn next(&mut self) -> core::prelude::v1::Option<Self::Item> {
        loop {
            let bit_index = self.current_u64.trailing_zeros() as u8;
            if bit_index == 64 {
                if self.offset == 64 * 3 {
                    return None;
                }
                self.offset += 64;
                self.current_u64 = self.bits[self.offset as usize / 64];
                continue;
            }
            self.current_u64 ^= 1 << bit_index;
            return Some(self.offset + bit_index);
        }
    }
}
