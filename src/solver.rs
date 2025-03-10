use std::{
    iter::zip,
    simd::prelude::*,
    time::{Duration, Instant},
};

use crate::{
    SIZE,
    bit_board::BitBoard,
    board::{Board, Cell, PLACED_SHIPS},
    ship_counts, step,
};
use num_format::{Locale, ToFormattedString};
use rand::random;
use rayon::prelude::*;

pub struct PlacedBitShips {
    pub placed_ships: [[BitBoard; 256]; 4],
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
    pub placed_bit_ships: PlacedBitShips,
    current_board: Board,
}

impl Solver {
    pub fn new() -> Self {
        Solver {
            placed_bit_ships: Self::gen_placed_bit_boards(),
            current_board: Board::new(),
        }
    }
    fn gen_placed_bit_boards() -> PlacedBitShips {
        let mut placed_ships = [[BitBoard::new(Board::new()); 256]; 4];
        for ship_i in 0..PLACED_SHIPS.len() {
            for dir in 0..2 {
                for i in 0..128 {
                    let bit_board_offset = Board::map_index_to_bit_index(i);
                    let bit_board_index = dir * 128 + bit_board_offset;

                    let index = dir * 128 + i;
                    if bit_board_index < 256 {
                        placed_ships[ship_i][bit_board_index] =
                            BitBoard::new(PLACED_SHIPS[ship_i][index]);
                    }
                }
            }
        }
        PlacedBitShips { placed_ships }
    }
    pub fn reset(&mut self) {
        self.current_board = Board::new();
    }
    pub fn run(&self, time_to_run: Duration) {
        let start_time = Instant::now();
        let ship_counts = self.inner_loop(time_to_run);

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
        println!("ship_counts: {}", ship_counts);
        println!(
            "total_ships: {}",
            ship_counts.counts.iter().sum::<u64>() as f64 / ship_counts.board_count as f64
        );

        let x = max_index % SIZE;
        let y = max_index / SIZE;

        println!("Max (x, y): ({}, {})", (x as u8 + b'A') as char, y + 1);
    }
    pub fn inner_loop(&self, time_to_run: Duration) -> ship_counts::ShipCounts {
        // pub fn inner_loop(&mut self, random_values: &[[u32; SHIPS.len()]]) -> ship_counts::ShipCounts {
        let bit_board = BitBoard::new(self.current_board);

        (0..rayon::current_num_threads())
            .par_bridge()
            .map(|_| {
                let mut ship_counts = ship_counts::ShipCounts::new();
                let mut special_rng = SpecialRng::new();
                let ship_amounts = std::hint::black_box([4, 3, 2, 1]);

                let end_time = Instant::now() + time_to_run;
                while Instant::now() < end_time {
                    step(
                        bit_board,
                        ship_amounts,
                        &mut ship_counts,
                        &self.placed_bit_ships,
                        &mut special_rng,
                    );
                }
                ship_counts
            })
            .reduce(ship_counts::ShipCounts::new, |mut a, b| {
                a.add_other_count(b);
                a
            })
    }
}
