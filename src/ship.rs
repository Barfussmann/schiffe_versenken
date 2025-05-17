use std::marker::ConstParamTy;
#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub enum ShipLength {
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub struct Ship {
    pub length: ShipLength,
    pub index: usize,
}

const LENGTHS: [usize; 5] = [5, 4, 3, 3, 2];
impl Ship {
    pub fn length_from_index(index: usize) -> usize {
        LENGTHS[index]
    }
    pub const fn new(length: usize, index: usize) -> Ship {
        let length = match length {
            1 => ShipLength::_1,
            2 => ShipLength::_2,
            3 => ShipLength::_3,
            4 => ShipLength::_4,
            5 => ShipLength::_5,
            6 => ShipLength::_6,
            _ => unreachable!(),
        };
        Ship { length, index }
    }
    pub const fn index(self) -> usize {
        self.index
    }
    pub const fn length(self) -> usize {
        self.length as usize
    }
}
