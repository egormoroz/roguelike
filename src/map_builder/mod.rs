use macroquad::prelude::IVec2;
use specs::World;
use crate::{map::*, spawner::Spawner};

mod simple;
pub use simple::*;

pub trait MapBuilder {
    fn generate(&mut self);
    fn player_pos(&self) -> IVec2;
    fn spawn(&self, ecs: &mut World, spawner: &mut Spawner);
    fn build(self) -> Map;
}

