use std::io::Write;
use macroquad::prelude::IVec2;
use smallvec::SmallVec;
use specs::prelude::*;
use crate:: {
    comp::*, 
    util::{GameLog, to_cp437, colors::*}, 
    map::Map,
    alg::compute_fov,
    systems::ParticleBuilder
};


pub struct InventorySystem;

impl<'a> System<'a> for InventorySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Named>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player_entity, mut log, named, mut wants_pickup, mut positions, mut backpacks) = data;

        for (entity, pickup) in (&entities, &wants_pickup).join() {
            positions.remove(pickup.item);
            backpacks.insert(pickup.item, InBackpack { owner: entity })
                .expect("failed to insert backpack entry");
            
            if entity == *player_entity {
                write!(log.new_entry(), "You pick up the {}.", 
                    named.get(pickup.item).unwrap().0).unwrap();
            }
        }

        wants_pickup.clear();
    }
}

#[derive(Default)]
pub struct ItemUseSystem {
    aoe_cache: SmallVec<[(i32, i32); 256]>,
    target_cache: SmallVec<[Entity; 256]>,
}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Named>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, AreaOfEffect>,
        ReadStorage<'a, Equippable>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, WantsToUseItem>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, map, player_entity, mut log, mut particle_builder,
            named, healers, inflicts_damage, 
            consumables, aoe, equippable, 
            positions, mut confused, 
            mut suffer_damage, mut wants_use, mut stats, 
            mut equipped, mut backpacked) = data;
        let player_entity = *player_entity;

        for (user, useitem, stats) in (&entities, &wants_use, &mut stats).join() {
            let mut used = false;
            self.target_cache.clear();

            if let UseTarget::Point(center) = useitem.target {
                self.aoe_cache.clear();
                let is_aoe;
                if let Some(aoe) = aoe.get(useitem.item) {
                    compute_fov(IVec2::new(center.0, center.1), aoe.radius, 
                        &*map, |tile| self.aoe_cache.push((tile.x, tile.y)));
                    self.aoe_cache.sort_unstable();
                    self.aoe_cache.dedup();
                    is_aoe = true;
                } else {
                    self.aoe_cache.push(center);
                    is_aoe = false;
                }
                
                for (x, y) in self.aoe_cache.iter().cloned() {
                    // if !map.bounds().contains(x, y) { continue; }
                    for victim in map.tile_content(x, y) {
                        self.target_cache.push(*victim);
                    }
                    if is_aoe {
                        particle_builder.request(x, y, to_cp437('░'), ORANGE, BLACK, 200.);
                    }
                }
            } else if let UseTarget::User = useitem.target {
                self.target_cache.push(user);
            }

            for target in self.target_cache.iter() {
                if let Some(dmg) = inflicts_damage.get(useitem.item) {
                    SufferDamage::new_damage(&mut suffer_damage, *target, dmg.damage);
                    if user == player_entity {
                        let target_name = &named.get(*target).unwrap().0;
                        let item_name = &named.get(useitem.item).unwrap().0;
                        write!(log.new_entry(), "You use {} on {}, inflicting {} damage.", 
                            item_name, target_name, dmg.damage).unwrap()
                    }
                    used = true;

                    if let Some(pos) = positions.get(*target) {
                        particle_builder.request(pos.x, pos.y, to_cp437('‼'), RED, BLACK, 200.);
                    }
                }

                if let Some(confusion) = confused.get(useitem.item).cloned() {
                    confused.insert(*target, confusion).expect("failed to insert confusion");
                    if user == player_entity {
                        let target_name = &named.get(*target).unwrap().0;
                        let item_name = &named.get(useitem.item).unwrap().0;
                        write!(log.new_entry(), "You use {} on {}, confusing them.",
                            item_name, target_name).unwrap();
                    }
                    used = true;

                    if let Some(pos) = positions.get(*target) {
                        particle_builder.request(pos.x, pos.y, to_cp437('?'), MAGENTA, BLACK, 200.);
                    }
                }

                if let Some(healer) = healers.get(useitem.item) {
                    used = true;
                    stats.hp = stats.max_hp.min(stats.hp + healer.heal_amount);
                    if user == player_entity {
                        let name = &named.get(useitem.item).unwrap().0;
                        write!(log.new_entry(), "You drink the {}, healing {} hp.", 
                            name, healer.heal_amount).unwrap();
                    }

                    if let Some(pos) = positions.get(user) {
                        particle_builder.request(pos.x, pos.y, to_cp437('♥'), GREEN, BLACK, 200.);
                    }
                } 
            
                if let Some(Equippable { slot }) = equippable.get(useitem.item) {
                    let slot = *slot;
                    let mut to_unequip = SmallVec::<[Entity; 4]>::new();
                    for (itm, equipped) in (&entities, &equipped).join() {
                        if equipped.owner == *target && equipped.slot == slot {
                            to_unequip.push(itm);
                            if equipped.owner == player_entity {
                                let name = &named.get(itm).unwrap().0;
                                write!(log.new_entry(), "You unequip {}.", name).unwrap();
                            }
                        }
                    }

                    for e in to_unequip {
                        equipped.remove(e);
                        backpacked.insert(e, InBackpack { owner: *target })
                            .expect("failed to insert InBackpack");
                    }

                    backpacked.remove(useitem.item).expect("failed to remove InBackpack");
                    equipped.insert(useitem.item, Equipped { owner: *target, slot })
                        .expect("failed to insert Equipped");
                    if *target == player_entity {
                        let name = &named.get(useitem.item).unwrap().0;
                        write!(log.new_entry(), "You equip {}.", name).unwrap();
                    }
                } 
            }

            if used && consumables.contains(useitem.item) {
                entities.delete(useitem.item).expect("delete failed");
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem;

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Named>,
        WriteStorage<'a, WantsToDropItem>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player_entity, mut log, named, mut wants_drop, mut positions, mut backpacked) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let pos = *positions.get(entity).unwrap();
            positions.insert(to_drop.item, pos).expect("failed to insert position");
            backpacked.remove(to_drop.item);

            if entity == *player_entity {
                write!(log.new_entry(), "You drop the {}.", 
                    named.get(entity).unwrap().0).unwrap();
            }
        }

        wants_drop.clear();
    }
}
