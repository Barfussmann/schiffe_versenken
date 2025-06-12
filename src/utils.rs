use glam::IVec2;

use crate::SIZE;

/// Returns an iterator over all points in a rectangle.
/// clamps the coords to board size.
/// Bottom right corner is exclusive.
pub fn rect_iter(top_left: IVec2, bottom_right: IVec2) -> impl Iterator<Item = IVec2> {
    const BOARD_SIZE: IVec2 = IVec2::splat(SIZE as i32);
    let clamped_top_left = top_left.clamp(IVec2::ZERO, BOARD_SIZE);
    let clamped_bottom_right = bottom_right.clamp(IVec2::ZERO, BOARD_SIZE);

    (clamped_top_left.x..clamped_bottom_right.x).flat_map(move |x| {
        (clamped_top_left.y..clamped_bottom_right.y).map(move |y| IVec2::new(x, y))
    })
}
