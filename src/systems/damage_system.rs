use specs::prelude::*;
use crate::{comp::*, util::GameLog, state::RunState};
use std::io::Write;

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }
        damage.clear();
    }
}

impl DamageSystem {
    pub fn delete_the_dead(ecs: &mut World) {
        let mut dead = vec![];
        {
            let stats = ecs.read_storage::<CombatStats>();
            let players = ecs.read_storage::<Player>();
            let names = ecs.read_storage::<Named>();
            let entities = ecs.entities();
            let mut log = ecs.write_resource::<GameLog>();

            for (entity, stats, name) in (&entities, &stats, &names).join() {
                if stats.hp <= 0 {
                    dead.push(entity);
                    match players.get(entity) {
                        Some(_) => *ecs.fetch_mut::<RunState>() = RunState::GameOver,
                        None => write!(log.new_entry(), "{} dies.", name.0).unwrap(),
                    };
                }
            }
        };

        ecs.delete_entities(&dead).unwrap();
    }
}
