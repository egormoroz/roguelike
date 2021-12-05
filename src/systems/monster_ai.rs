use macroquad::prelude::IVec2;
use specs::prelude::*;

use crate::{
    comp::*, 
    map::{Map, ViewMap}, 
    util::{IRect, to_cp437, colors::*, DjMap},
    state::RunState,
    systems::ParticleBuilder,
};

#[derive(Default)]
pub struct MonsterAI;

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, DjMap>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, IVec2>,
        ReadExpect<'a, RunState>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, EntityMoved>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, dj_map, player, plp, state,
            mut map, mut particle_builder, monster, 
            mut confused, mut viewshed, mut pos, 
            mut wants_to_melee, mut entity_moved) = data;

        match *state {
            RunState::MonsterTurn => (),
            _ => return,
        };

        for (entity, viewshed, pos, _) in (&entities, &mut viewshed, &mut pos, &monster).join() {
            if let Some(confusion) = confused.get_mut(entity) {
                confusion.turns -= 1;
                if confusion.turns <= 0 {
                    confused.remove(entity);
                }
                particle_builder.request(pos.x, pos.y, to_cp437('?'), MAGENTA, BLACK, 200.);
                continue;
            }

            if viewshed.can_see(plp.x, plp.y) {
                if IRect::new(pos.x - 1, pos.y - 1, 3, 3).contains(plp.x, plp.y) {
                    wants_to_melee.insert(entity, WantsToMelee { target: *player }).unwrap();
                    continue;
                }
                let dst = |x: i32, y: i32| (x - plp.x) * (x - plp.x) 
                    + (y - plp.y) * (y - plp.y);

                let step = dj_map.adjacent(pos.x, pos.y)
                    .filter(|(x, y, _)| !map.tile_flags(*x, *y).blocked)
                    .min_by(|(x1, y1, d1), (x2, y2, d2)| d1.cmp(d2)
                        .then(dst(*x1, *y1).cmp(&dst(*x2, *y2))));

                if let Some((x, y, _)) = step {
                    map.tile_flags_mut(pos.x, pos.y).blocked = false;
                    map.tile_flags_mut(x, y).blocked = true;
                    pos.x = x; pos.y = y;
                    viewshed.dirty = true;
                    entity_moved.insert(entity, EntityMoved {}).expect("failed to insert EntityMoved");
                }
            }
        }
    }
}

