use rand::{thread_rng, Rng};
use specs::{prelude::*, saveload::{MarkedBuilder, SimpleMarker}};
use smallvec::smallvec;
use super::{
    comp::*,
    util::IRect,
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
}

pub fn player(ecs: &mut World, x: i32, y: i32) -> Entity {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable { 
            glyph: to_cp437('@'), 
            fg: YELLOW, 
            bg: BLACK,
            order: 0,
        })
        .with(Viewshed { range: 8, visible_tiles: smallvec![], dirty: true })
        .with(Player{})
        .with(Named("Player".to_owned()))
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

const MAX_DEPTH1_SPAWNS: i32 = 4;

pub struct RoomSpawner {
    spawn_points: Vec<(i32, i32)>,
    table: RandomTable<SpawnOption>,
    depth: i32,
}

impl RoomSpawner {
    pub fn new(depth: i32) -> Self {
        let mut inst = Self { 
            spawn_points: vec![], 
            table: RandomTable::new(), 
            depth 
        };
        inst.update_table();
        inst
    }

    pub fn spawn(&mut self, ecs: &mut World, room: &IRect) {
        let mut rng = thread_rng();
        let num_spawns = rng.gen_range(1..=MAX_DEPTH1_SPAWNS + self.depth);
        self.spawn_points.clear();
        self.spawn_points.reserve(num_spawns as usize);

        for _ in 1..=num_spawns {
            loop {
                let x = rng.gen_range(room.x..=room.xx);
                let y = rng.gen_range(room.y..=room.yy);
                if !self.spawn_points.contains(&(x, y)) { 
                    self.spawn_points.push((x, y));
                    break;
                }
            }
        }

        for (x, y) in self.spawn_points.iter().cloned() {
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
            }
        }
    }

    pub fn set_depth(&mut self, depth: i32) {
        self.depth = depth;
        self.update_table();
    }

    fn update_table(&mut self) {
        use SpawnOption::*;
        self.table.clear();
        let d = self.depth;
        let weights = [(Goblin, 10), (Orc, 1 + d),
            (HealthPotion, 7), (FireballScroll, 2 + d), (ConfusionScroll, 2 + d),
            (MagicMissileScroll, 4), (Dagger, 3), (Shield, 3)];
        self.table.extend(weights.into_iter());
    }
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

pub fn dagger(ecs: &mut World, x: i32, y: i32) {
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
        .with(CombatBonuses { power: 2, defense: 0 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn shield(ecs: &mut World, x: i32, y: i32) {
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
        .with(CombatBonuses { power: 0, defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
