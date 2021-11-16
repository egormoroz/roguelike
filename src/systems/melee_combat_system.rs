use specs::prelude::*;
use crate::{comp::*, util::GameLog};
use super::ParticleBuilder;


pub struct MeleeCombatSystem;

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        Write<'a, ParticleBuilder>,
        ReadStorage<'a, Named>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, AttackBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, SufferDamage>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut particle_builder, names, 
            combat_stats, attack_bonuses, defense_bonuses, 
            equipped, positions, mut log, 
            mut wants_melee, mut inflict_damage) = data;

        for (attacker, name, stats, wants_melee) 
            in (&entities, &names, &combat_stats, &mut wants_melee).join() 
        {
            if stats.hp <= 0 { continue; }
            if let Some(pos) = positions.get(wants_melee.target) {
                use crate::util::{colors::*, to_cp437};
                particle_builder.request(pos.x, pos.y, to_cp437('‼'), ORANGE, BLACK, 100.)
            }

            let (mut offensive_bonus, mut defensive_bonus) = (0, 0);
            for (bonus, equipped) in (&attack_bonuses, &equipped).join() {
                if equipped.owner == attacker {
                    offensive_bonus += bonus.power;
                }
            }

            for (bonus, equipped) in (&defense_bonuses, &equipped).join() {
                if equipped.owner == wants_melee.target {
                    defensive_bonus += bonus.defense;
                }
            }

            let target_stats = combat_stats.get(wants_melee.target).unwrap();
            if target_stats.hp <= 0 { continue; }
            let target_name = names.get(wants_melee.target).unwrap();
            let damage = stats.power + offensive_bonus 
                - (target_stats.defense + defensive_bonus);
            
            use std::io::Write;
            let mut entry = log.new_entry();
            if damage > 0 {
                SufferDamage::new_damage(
                    &mut inflict_damage, 
                    wants_melee.target, 
                    damage
                );
                write!(entry, "{} hits {} for {} hp.", name.0, target_name.0, damage).unwrap();
            } else {
                write!(entry, "{} is unable to hurt {}.", name.0, target_name.0).unwrap();
            }
        }

        wants_melee.clear();
    }
}
