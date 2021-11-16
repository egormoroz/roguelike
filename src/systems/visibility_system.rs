use std::io::Write;
use rand::{thread_rng, Rng};
use crate::{
    specs::prelude::*,
    comp::*, 
    map::Map, 
    alg::compute_fov,
    util::GameLog,
};

pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Named>,
        WriteStorage<'a, Viewshed>, 
        WriteStorage<'a, Hidden>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut map, mut log,
            pos, players, names, 
            mut viewshed, mut hidden) = data;
        let mut rng = thread_rng();

        for (ent,viewshed,pos) in (&entities, &mut viewshed, &pos).join() {
            if !viewshed.dirty { continue; }
            viewshed.dirty = false;
            viewshed.visible_tiles.clear();
            compute_fov((*pos).into(), viewshed.range, &*map, 
                |tile| viewshed.visible_tiles.push((tile.x, tile.y)));
            viewshed.sort_dedup();

            if players.contains(ent) {
                map.reset_visible_tiles();
                for (x, y) in &viewshed.visible_tiles {
                    map.tile_flags_mut(*x, *y).revealed = true;
                    map.tile_flags_mut(*x, *y).visible = true;

                    for e in map.tile_content(*x, *y) {
                        if !hidden.contains(*e) { continue; }
                        if rng.gen_range(1..24) != 1 { continue; }

                        if let Some(name) = names.get(*e) {
                            write!(log.new_entry(), "You spotted a {}.", &name.0).unwrap();
                        }
                        hidden.remove(*e);
                    }
                }
            }
        }
    }
}
