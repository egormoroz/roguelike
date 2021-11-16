use super::util::Glyph;
use macroquad::prelude::IVec2;

use smallvec::{SmallVec, smallvec};

use specs::{
    saveload::*,
    prelude::*,
};

#[allow(deprecated)]
use specs::error::NoError;

use specs_derive::*;
use serde::{Serialize, Deserialize};

pub fn register_all_components(ecs: &mut World) {
    ecs.register::<Position>();
    ecs.register::<Renderable>();
    ecs.register::<Player>();
    ecs.register::<Viewshed>();
    ecs.register::<Monster>();
    ecs.register::<Named>();
    ecs.register::<BlocksTile>();
    ecs.register::<CombatStats>();
    ecs.register::<WantsToMelee>();
    ecs.register::<SufferDamage>();
    ecs.register::<Item>();
    ecs.register::<ProvidesHealing>();
    ecs.register::<InBackpack>();
    ecs.register::<WantsToPickupItem>();
    ecs.register::<WantsToUseItem>();
    ecs.register::<WantsToDropItem>();
    ecs.register::<Consumable>();
    ecs.register::<Ranged>();
    ecs.register::<InflictsDamage>();
    ecs.register::<AreaOfEffect>();
    ecs.register::<Confusion>();
    ecs.register::<Equippable>();
    ecs.register::<Equipped>();
    ecs.register::<AttackBonus>();
    ecs.register::<DefenseBonus>();
    ecs.register::<ParticleLifetime>();
    ecs.register::<HungerClock>();
    ecs.register::<Nutritious>();

    ecs.register::<SimpleMarker<SerializeMe>>();
}

#[derive(Default, Component, ConvertSaveload, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Into<IVec2> for Position {
    fn into(self) -> IVec2 {
        IVec2::new(self.x, self.y)
    }
}

impl From<IVec2> for Position {
    fn from(v: IVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

#[derive(Default, Component, ConvertSaveload, Clone, Copy)]
pub struct Renderable {
    pub glyph: Glyph,
    pub fg: [f32; 4],
    pub bg: [f32; 4],
    pub order: i32,
}

#[derive(Component, Default, Serialize, Deserialize, Clone, Copy)]
#[storage(NullStorage)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: SmallVec<[(i32, i32); 256]>,
    pub range: i32,
    pub dirty: bool,
}

impl Viewshed {
    pub fn sort_dedup(&mut self) {
        self.visible_tiles.sort_unstable();
        self.visible_tiles.dedup();
    }

    pub fn can_see(&self, x: i32, y: i32) -> bool {
        self.visible_tiles.binary_search(&(x, y)).is_ok()
    }
}

#[derive(Component, Default, Serialize, Deserialize, Clone, Copy)]
#[storage(NullStorage)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Named(pub String);


#[derive(Component, Default, Serialize, Deserialize, Clone, Copy)]
#[storage(NullStorage)]
pub struct BlocksTile {}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: SmallVec<[i32; 8]>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount: smallvec![amount] };
            store.insert(victim, dmg).unwrap();
        }
    }
}

#[derive(Component, Default, Serialize, Deserialize, Clone, Copy)]
#[storage(NullStorage)]
pub struct Item {}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct WantsToPickupItem {
    pub item: Entity,
}


#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum UseTarget {
    User,
    Point((i32, i32))
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: UseTarget,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Default, Serialize, Deserialize, Clone, Copy)]
#[storage(NullStorage)]
pub struct Consumable {}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Clone, Copy, ConvertSaveload)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot { MainHand, OffHand }

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct AttackBonus {
    pub power: i32,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct DefenseBonus {
    pub defense: i32,
}

#[derive(Default, Component, Clone, Copy)]
pub struct ParticleLifetime {
    pub remaining_ms: f32,
}

#[derive(Component, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum HungerState { WellFed, Normal, Hungry, Starving }

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

#[derive(Component, Default, Serialize, Deserialize, Clone, Copy)]
#[storage(NullStorage)]
pub struct Nutritious {}

pub struct SerializeMe {}
