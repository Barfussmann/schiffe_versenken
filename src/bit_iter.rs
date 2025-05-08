use std::simd::u64x4;

#[derive(Clone, Copy)]
pub struct BitIter {
    current_u64: u64,
    bits: u64x4,
    offset: u8,
}
impl BitIter {
    // #[inline(never)]
    pub fn new(bits: u64x4) -> Self {
        Self {
            current_u64: bits[0],
            bits,
            offset: 0,
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
