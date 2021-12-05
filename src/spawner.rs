use specs::{prelude::*, saveload::{MarkedBuilder, SimpleMarker}};
use smallvec::smallvec;
use super::{
    comp::*,
    util::Glyph,
    util::to_cp437,
    util::colors::*,
    random_table::RandomTable,
};

#[derive(Debug, Clone, Copy)]
enum SpawnOption {
    Goblin,
    Orc,
    HealthPotion,
    FireballScroll,
    ConfusionScroll,
    MagicMissileScroll,
    Dagger,
    Shield,
    LongSword,
    TowerShield,
    Rations,
    MagicMappingScroll,
    BearTrap,
}

pub struct Spawner {
    table: RandomTable<SpawnOption>,
    depth: i32,
}

impl Spawner {
    pub fn new(depth: i32) -> Self {
        let mut inst = Self { 
            table: RandomTable::new(), 
            depth 
        };
        inst.update_table();
        inst
    }

    pub fn spawn(&mut self, ecs: &mut World, x: i32, y: i32) {
        use SpawnOption::*;
        match *self.table.roll() {
            Goblin => goblin(ecs, x, y),
            Orc => orc(ecs, x, y),
            HealthPotion => health_potion(ecs, x, y),
            FireballScroll => fireball_scroll(ecs, x, y),
            ConfusionScroll => confusion_scroll(ecs, x, y),
            MagicMissileScroll => magic_missile_scroll(ecs, x, y),
            Dagger => dagger(ecs, x, y),
            Shield => shield(ecs, x, y),
            LongSword => longsword(ecs, x, y),
            TowerShield => tower_shield(ecs, x, y),
            Rations => rations(ecs, x, y),
            MagicMappingScroll => magic_mapping_scroll(ecs, x, y),
            BearTrap => bear_trap(ecs, x, y),
        }
    }

    pub fn set_depth(&mut self, depth: i32) {
        if depth == self.depth { return; }
        self.depth = depth;
        self.update_table();
    }

    fn update_table(&mut self) {
        use SpawnOption::*;
        let d = self.depth;
        let weights = [
            (Goblin, 100), (Orc, 1 + d),
            (HealthPotion, 7), (FireballScroll, 2 + d), (ConfusionScroll, 2 + d),
            (MagicMissileScroll, 4), (Dagger, 3), (Shield, 3),
            (LongSword, d - 1), (TowerShield, d - 1), (Rations, 10),
            (MagicMappingScroll, 2), (BearTrap, 5),
        ];
        self.table.clear();
        self.table.extend(weights.into_iter());
    }
}

pub fn player(ecs: &mut World, x: i32, y: i32) -> Entity {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable { 
            glyph: to_cp437('@'), 
            fg: YELLOW, 
            bg: BLACK,
            order: 1,
        })
        .with(Viewshed { range: 8, visible_tiles: smallvec![], dirty: true })
        .with(Player{})
        .with(Named("Player".to_owned()))
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 20, power: 5 })
        .with(HungerClock { state: HungerState::WellFed, duration: 20 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, to_cp437('o'), "Orc".to_owned())
}

fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, to_cp437('g'), "Goblin".to_owned())
}

fn monster(ecs: &mut World, x: i32, y: i32, glyph: Glyph, name: String) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph, 
            fg: RED, 
            bg: BLACK,
            order: 1,
        })
        .with(Viewshed { range: 8, visible_tiles: smallvec![], dirty: true })
        .with(Monster {})
        .with(Named(name))
        .with(BlocksTile {})
        .with(CombatStats { max_hp: 16, hp: 16, defense: 1, power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('¡'),
            fg: MAGENTA,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Health potion".to_owned()))
        .with(Item {})
        .with(ProvidesHealing { heal_amount: 8 })
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y})
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: CYAN,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Magic missile scroll".to_owned()))
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y})
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: ORANGE,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Fireball scroll".to_owned()))
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y})
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: PINK,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Confusion scroll".to_owned()))
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y})
        .with(Renderable {
            glyph: to_cp437('/'),
            fg: CYAN,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Dagger".to_owned()))
        .with(Item{})
        .with(Equippable { slot: EquipmentSlot::MainHand })
        .with(AttackBonus { power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y})
        .with(Renderable {
            glyph: to_cp437('('),
            fg: CYAN,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Shield".to_owned()))
        .with(Item{})
        .with(Equippable { slot: EquipmentSlot::OffHand })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y})
        .with(Renderable {
            glyph: to_cp437('/'),
            fg: YELLOW,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Longsword".to_owned()))
        .with(Item{})
        .with(Equippable { slot: EquipmentSlot::MainHand })
        .with(AttackBonus { power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y})
        .with(Renderable {
            glyph: to_cp437('('),
            fg: YELLOW,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Tower shield".to_owned()))
        .with(Item{})
        .with(Equippable { slot: EquipmentSlot::OffHand })
        .with(DefenseBonus { defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn rations(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y})
        .with(Renderable {
            glyph: to_cp437('%'),
            fg: GREEN,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Rations".to_string()))
        .with(Item {})
        .with(Nutritious {})
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_mapping_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y})
        .with(Renderable { 
            glyph: to_cp437(')'),
            fg: CYAN,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Scroll of Magic Mapping".to_owned()))
        .with(Item {})
        .with(MagicMapper {})
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('^'),
            fg: RED,
            bg: BLACK,
            order: 2,
        })
        .with(Named("Bear trap".to_owned()))
        .with(Hidden {})
        .with(EntryTrigger {})
        .with(InflictsDamage { damage: 6 })
        .with(SingleActivation {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

