use std::marker::ConstParamTy;

pub const SHIPS: [Ship; 4] = [Ship::new(5), Ship::new(4), Ship::new(3), Ship::new(2)];

#[derive(Clone, Copy)]
pub struct ShipCounts {
    counts: [usize; 4],
}
impl ShipCounts {
    pub fn new(counts: [usize; 4]) -> Self {
        assert!(counts[0] <= 1);
        assert!(counts[1] <= 1);
        assert!(counts[2] <= 2);
        assert!(counts[3] <= 1);
        Self { counts }
    }
    pub fn counts(&self) -> [usize; 4] {
        self.counts
    }
    pub fn iter_ships(&self) -> impl Iterator<Item = Ship> {
        self.counts
            .iter()
            .enumerate()
            .flat_map(|(ship_index, ship_count)| {
                std::iter::repeat_n(unsafe { *SHIPS.get_unchecked(ship_index) }, *ship_count)
            })
    }
}

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
}

impl Ship {
    pub fn from_index(index: usize) -> Ship {
        SHIPS[index]
    }
    pub const fn new(length: usize) -> Ship {
        let length = match length {
            1 => ShipLength::_1,
            2 => ShipLength::_2,
            3 => ShipLength::_3,
            4 => ShipLength::_4,
            5 => ShipLength::_5,
            6 => ShipLength::_6,
            _ => unreachable!(),
        };
        Ship { length }
    }
    pub const fn length(self) -> usize {
        self.length as usize
    }
}
