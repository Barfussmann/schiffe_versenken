use std::simd::u64x4;

pub struct ChunkedBitIter {
    current_u64: u64,
    bits: u64x4,
    offset: u8,
}

impl ChunkedBitIter {
    pub const CHUNK_SIZE: usize = 8;
    pub fn new(bits: u64x4) -> Self {
        Self {
            current_u64: bits[0],
            bits,
            offset: 0,
        }
    }
}
impl Iterator for ChunkedBitIter {
    type Item = [u8; Self::CHUNK_SIZE];

    // #[inline(never)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_u64.trailing_zeros() == 64 {
                if self.offset == 64 * 3 {
                    return None;
                }
                self.offset += 64;
                self.current_u64 = self.bits[self.offset as usize / 64];
                continue;
            }

            let mut ret = [0; Self::CHUNK_SIZE];

            for i in 0..Self::CHUNK_SIZE {
                ret[i] = self.current_u64.trailing_zeros() as u8;
                self.current_u64 ^= 1_u64.unbounded_shl(ret[i] as u32);
            }

            for val in &mut ret {
                *val += self.offset;
            }

            return Some(ret);
        }
    }
}

#[derive(Clone, Copy)]
pub struct BitIter {
    current_u64: u64,
    bits: u64x4,
    offset: u8,
}
impl BitIter {
    pub fn new(bits: u64x4) -> Self {
        Self {
            current_u64: bits[0],
            bits,
            offset: 0,
        }
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
