use macroquad::prelude::IVec2;
use specs::prelude::*;

use crate::{
    comp::*, 
    map::Map, 
    util::IRect,
    state::RunState,
    alg::AStarPath,
};

#[derive(Default)]
pub struct MonsterAI {
    pf_cache: AStarPath,
}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, IVec2>,
        ReadExpect<'a, RunState>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player, plp, state,
            mut map, monster, mut confused, mut viewshed, 
            mut pos, mut wants_to_melee) = data;

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
                continue;
            }

            if viewshed.can_see(plp.x, plp.y) {
                if IRect::new(pos.x - 1, pos.y - 1, 3, 3).contains(plp.x, plp.y) {
                    wants_to_melee.insert(entity, WantsToMelee { target: *player }).unwrap();
                    continue;
                }

                self.pf_cache.compute(&*map, (*pos).into(), *plp);
                let step = self.pf_cache.result().iter().rev().skip(1).next();
                if let Some((step, _)) = step.cloned() {
                    map.tile_flags_mut(pos.x, pos.y).blocked = false;
                    map.tile_flags_mut(step.x, step.y).blocked = true;
                    *pos = step.into();
                    viewshed.dirty = true;
                }
            }
        }
    }
}
