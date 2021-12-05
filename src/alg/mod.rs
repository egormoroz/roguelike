mod astar;
mod fov;
mod bfs;

pub use astar::*;
pub use fov::*;
pub use bfs::*;

use macroquad::prelude::IVec2;
use smallvec::SmallVec;

pub trait BaseMap {
    fn size(&self) -> IVec2;
    fn is_opaque(&self, pos: IVec2) -> bool;

    fn distance(&self, a: IVec2, b: IVec2) -> f32 {
        ((b - a).dot(b - a) as f32).sqrt()
    }

    fn successors(&self, pos: IVec2) -> SmallVec<[(IVec2, f32); 8]>;
}
