use std::io::Write;
use specs::prelude::*;
use crate::{
    comp::*,
    util::{GameLog, to_cp437, colors::*},
    map::Map,
    systems::ParticleBuilder,
};

pub struct TriggerSystem;

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, Named>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, SingleActivation>,
        WriteStorage<'a, EntityMoved>,
        WriteStorage<'a, Hidden>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, map, mut log, mut particle_builder,
            positions, entry_triggers, names, 
            inflicts_damage, single_activation, mut entity_moved, 
            mut hiddens, mut suffer_damage) = data;

        for (actor, pos, _) in (&entities, &positions, &entity_moved).join() {
            for reactor in map.tile_content(pos.x, pos.y) {
                if actor == *reactor { continue; }
                if !entry_triggers.contains(*reactor) { continue; }

                if let Some(name) = names.get(*reactor) {
                    write!(log.new_entry(), "{} triggers.", &name.0).unwrap();
                }
                if let Some(inflicts) = inflicts_damage.get(*reactor) {
                    particle_builder.request(pos.x, pos.y, to_cp437('‼'), ORANGE, BLACK, 200.);
                    SufferDamage::new_damage(&mut suffer_damage, actor, inflicts.damage);
                }
                if single_activation.contains(*reactor) {
                    entities.delete(*reactor).expect("failed to delete reactor");
                }

                hiddens.remove(*reactor);
            }
        }

        entity_moved.clear();
    }
}
