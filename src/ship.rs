use std::marker::ConstParamTy;

pub const SHIPS: [Ship; 4] = [Ship::new(5), Ship::new(4), Ship::new(3), Ship::new(2)];

#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub fn all_possible_ship_counts() -> impl Iterator<Item = Self> {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                for i_0 in 0..=1 {
                    for i_1 in 0..=1 {
                        for i_2 in 0..=2 {
                            for i_3 in 0..=1 {
                                yield Self::new([i_0, i_1, i_2, i_3])
                            }
                        }
                    }
                }
            },
        )
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
    pub fn total_ships(&self) -> usize {
        self.counts.into_iter().sum::<usize>()
    }
    pub fn remove_placed_ship(&self, placed_ship: Ship) -> Self {
        let mut removed = *self;

        removed.counts[placed_ship.index()] = removed.counts[placed_ship.index()]
            .checked_sub(1)
            .expect("Can't go lower than 0 Ships");
        removed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub enum ShipLength {
    // _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    // _6 = 6,
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
            // 1 => ShipLength::_1,
            2 => ShipLength::_2,
            3 => ShipLength::_3,
            4 => ShipLength::_4,
            5 => ShipLength::_5,
            // 6 => ShipLength::_6,
            _ => unreachable!(),
        };
        Ship { length }
    }
    pub const fn length(self) -> usize {
        self.length as usize
    }
    pub fn index(self) -> usize {
        SHIPS.iter().position(|s| *s == self).unwrap()
    }
}
