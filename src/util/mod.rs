mod cp437;
mod irect;
mod gamelog;
mod grid;
mod djmap;
pub mod colors;


pub use cp437::*;
pub use irect::*;
pub use gamelog::*;
pub use grid::*;
pub use djmap::*;

use macroquad::prelude::KeyCode;

pub fn letter_to_option(kc: KeyCode) -> i32 {
    const A_CODE: u32 = KeyCode::A as u32;
    const Z_CODE: u32 = KeyCode::Z as u32;

    match kc as u32 {
        kc @ A_CODE..=Z_CODE => (kc - A_CODE) as i32,
        _ => -1
    }
}

#[derive(Default, Clone, Copy)]
pub struct DeltaTime(pub f32);


pub fn adjacent(x: i32, y: i32) -> impl Iterator<Item = (i32, i32)> {
    [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)]
        .into_iter()
        .map(move |(dx, dy)| (x + dx, y + dy))
}

