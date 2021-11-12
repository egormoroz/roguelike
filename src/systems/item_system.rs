use macroquad::prelude::IVec2;
use smallvec::SmallVec;
use specs::prelude::*;
use crate:: {
    comp::*, 
    util::GameLog, 
    map::Map,
    alg::compute_fov
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
                log.entries.push(format!("You pick up the {}.", named.get(pickup.item).unwrap().0));
            }
        }

        wants_pickup.clear();
    }
}

#[derive(Default)]
pub struct ItemUseSystem {
    aoe_cache: SmallVec<[(i32, i32); 256]>,
}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Named>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, WantsToUseItem>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, map, player_entity, mut log, named, 
            healers, inflicts_damage, consumables, aoe, 
            mut confused, mut suffer_damage, mut wants_use, mut stats) = data;

        for (user, useitem, stats) in (&entities, &wants_use, &mut stats).join() {
            let mut used = false;
             
            if let Some(center) = useitem.target {
                self.aoe_cache.clear();
                if let Some(aoe) = aoe.get(useitem.item) {
                    compute_fov(IVec2::new(center.0, center.1), aoe.radius, 
                        &*map, |tile| self.aoe_cache.push((tile.x, tile.y)));
                    self.aoe_cache.sort_unstable();
                    self.aoe_cache.dedup();
                } else {
                    self.aoe_cache.push(center);
                }
                
                for (x, y) in self.aoe_cache.iter().cloned() {
                    // if !map.bounds().contains(x, y) { continue; }
                    for victim in map.tile_content(x, y) {
                        if let Some(dmg) = inflicts_damage.get(useitem.item) {
                            SufferDamage::new_damage(&mut suffer_damage, *victim, dmg.damage);
                            if user == *player_entity {
                                let victim_name = &named.get(*victim).unwrap().0;
                                let item_name = &named.get(useitem.item).unwrap().0;
                                log.entries.push(format!("You use {} on {}, inflicting {} damage.", 
                                    item_name, victim_name, dmg.damage))
                            }
                            used = true;
                        }

                        if let Some(confusion) = confused.get(useitem.item).cloned() {
                            confused.insert(*victim, confusion).expect("failed to insert confusion");
                            if user == *player_entity {
                                let victim_name = &named.get(*victim).unwrap().0;
                                let item_name = &named.get(useitem.item).unwrap().0;
                                log.entries.push(format!("You use {} on {}, confusing them.",
                                    item_name, victim_name));
                            }
                            used = true;
                        }
                    }
                }
            } else if let Some(healer) = healers.get(useitem.item) {
                used = true;
                stats.hp = stats.max_hp.min(stats.hp + healer.heal_amount);
                if user == *player_entity {
                    let name = &named.get(useitem.item).unwrap().0;
                    log.entries.push(format!("You drink the {}, healing {} hp.", name, healer.heal_amount));
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
                log.entries.push(format!("You drop the {}.", 
                    named.get(entity).unwrap().0));
            }
        }

        wants_drop.clear();
    }
}
