use std::ops::{Index, IndexMut};

use crate::SIZE;

const GRID_SIZE: usize = SIZE.next_power_of_two();

#[derive(Clone, Copy, Debug)]
#[repr(align(128))]
pub struct CellGrid<T> {
    pub cells: [[T; GRID_SIZE]; GRID_SIZE],
}

const DIAGONAL_OFFSETS: [(i32, i32); 4] = [(-1, -1), (1, -1), (-1, 1), (1, 1)];
const ORTHOGONAL_OFFSETS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

impl<T: Copy> CellGrid<T> {
    pub const fn new(value: T) -> Self {
        Self {
            cells: [[value; GRID_SIZE]; GRID_SIZE],
        }
    }
    pub const fn is_inside(&self, x: usize, y: usize) -> bool {
        (x < SIZE) & (y < SIZE)
        // (0..SIZE).contains(&x) & (0..SIZE).contains(&y)
    }
    pub fn iter_offset_cords<const OFFSETS: [(i32, i32); 4]>(
        x: usize,
        y: usize,
    ) -> impl Iterator<Item = (usize, usize)> {
        OFFSETS
            .into_iter()
            .map(move |(x_off, y_off)| {
                let new_x = (x as i32 + x_off) as usize;
                let new_y = (y as i32 + y_off) as usize;
                (new_x, new_y)
            })
            .filter(|(x, y)| (0..SIZE).contains(x) & (0..SIZE).contains(y))
    }
    pub fn foreach_orthogonal_neighbors(&mut self, x: usize, y: usize, f: impl FnMut(&mut T)) {
        self.foreach_offset_neighbors::<ORTHOGONAL_OFFSETS>(x, y, f)
    }
    pub fn foreach_diagonal_neighbors(&mut self, x: usize, y: usize, f: impl FnMut(&mut T)) {
        self.foreach_offset_neighbors::<DIAGONAL_OFFSETS>(x, y, f)
    }
    pub fn foreach_offset_neighbors<const OFFSETS: [(i32, i32); 4]>(
        &mut self,
        x: usize,
        y: usize,
        mut f: impl FnMut(&mut T),
    ) {
        Self::iter_offset_cords::<OFFSETS>(x, y).for_each(|cords| f(&mut self[cords]));
    }
}

impl<T> Index<(usize, usize)> for CellGrid<T> {
    type Output = T;
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.cells[y][x]
    }
}
impl<T> IndexMut<(usize, usize)> for CellGrid<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.cells[y][x]
    }
}
