use std::{
    iter::zip,
    simd::u64x4,
    time::{Duration, Instant},
};

use crate::{
    SHIPS, SIZE,
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
    aes_key: u64x4,
    aes_value: u64x4,
}
impl SpecialRng {
    fn new() -> Self {
        Self {
            aes_key: u64x4::from_array(random()),
            aes_value: u64x4::splat(0),
        }
    }
    // #[inline(never)]
    pub fn wide_get_random(&mut self, upper_range: u64x4) -> u64x4 {
        let ret = unsafe {
            std::arch::x86_64::_mm256_mulhi_epu16(self.aes_value.into(), upper_range.into())
        }
        .into();
        unsafe {
            self.aes_value =
                std::arch::x86_64::_mm256_aesenc_epi128(self.aes_value.into(), self.aes_key.into())
                    .into();
        }
        ret
    }
    // #[inline(never)]
    pub fn get_random(&mut self, upper_range: u8) -> u32 {
        let ret = (self.aes_value[0] as u32)
            .widening_mul(upper_range as u32)
            .1;
        unsafe {
            self.aes_value =
                std::arch::x86_64::_mm256_aesenc_epi128(self.aes_value.into(), self.aes_key.into())
                    .into();
        }
        ret
        // self.small_rng.random_range(0..upper_range)
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
                    let bit_board_offset = BitBoard::map_index_to_bit_index(i);
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
    fn gen_random_values(length: u64) -> Box<[u32]> {
        let random_values: Box<[u32]> = (0..length * SHIPS.len() as u64)
            .into_par_iter()
            .map(|_| random())
            .collect();
        random_values
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
                    for _ in 0..100 {
                        // amortise the cost of the time comparison. Gives 10 % speedup
                        step(
                            bit_board,
                            ship_amounts,
                            &mut ship_counts,
                            &self.placed_bit_ships,
                            &mut special_rng,
                        );
                    }
                }
                ship_counts
            })
            .reduce(ship_counts::ShipCounts::new, |mut a, b| {
                a.add_other_count(b);
                a
            })
    }
}
