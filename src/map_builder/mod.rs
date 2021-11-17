use macroquad::prelude::IVec2;
use specs::World;
use crate::{
    map::*, 
    spawner::Spawner, 
    util::{Grid, IRect},
};

mod simple;
pub use simple::*;

const FLAGS: TileFlags = TileFlags {
    visible: true,
    revealed: true,
    blocked: false,
    bloodstained: false,
};

pub struct IntermediateMap<'a> {
    tiles: &'a Grid<TileType>,
}

pub trait MapBuilder {
    fn progress(&mut self) -> bool;
    fn spawn(&self, ecs: &mut World, spawner: &mut Spawner);

    fn player_pos(&self) -> IVec2;
    fn intermediate(&self) -> IntermediateMap;
    fn build(&mut self) -> Map;
}

impl<'a> ViewMap for IntermediateMap<'a> {
    fn bounds(&self) -> crate::util::IRect {
        IRect::new(0, 0, self.tiles.width(), self.tiles.height())
    }

    fn tile_flags(&self, _: i32, _: i32) -> &TileFlags {
        &FLAGS
    }

    fn tile(&self, x: i32, y: i32) -> &TileType {
        self.tiles.get(x, y)
    }
}
