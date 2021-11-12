use crate::{
    specs::prelude::*,
    comp::*, 
    map::Map, 
    alg::compute_fov,
};

pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Viewshed>, 
        WriteExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, pos, player, mut viewshed, mut map) = data;

        for (ent,viewshed,pos) in (&entities, &mut viewshed, &pos).join() {
            if !viewshed.dirty { continue; }
            viewshed.dirty = false;
            viewshed.visible_tiles.clear();
            compute_fov((*pos).into(), viewshed.range, &*map, 
                |tile| viewshed.visible_tiles.push((tile.x, tile.y)));
            viewshed.sort_dedup();

            if player.get(ent).is_some() {
                map.reset_visible_tiles();
                for (x, y) in &viewshed.visible_tiles {
                    map.tile_flags_mut(*x, *y).revealed = true;
                    map.tile_flags_mut(*x, *y).visible = true;
                }
            }
        }
    }
}
