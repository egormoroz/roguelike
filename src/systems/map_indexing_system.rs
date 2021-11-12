use specs::prelude::*;
use crate::{
    map::Map,
    comp::*,
};

pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, pos, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (entity, pos) in (&entities, &pos).join() {
            map.tile_flags_mut(pos.x, pos.y).blocked |= blockers.contains(entity);
            map.tile_content_mut(pos.x, pos.y).push(entity);
        }
    }
}
