use rand::{random, thread_rng, Rng};
use specs::{prelude::*, saveload::{MarkedBuilder, SimpleMarker}};
use super::{
    comp::*,
    util::IRect,
    util::Glyph,
    util::to_cp437,
    util::colors::*,
};
use smallvec::{SmallVec, smallvec};

const MAX_MONSTERS: i32 = 2;
const MAX_ITEMS: i32 = 4;

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

pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    match random() {
        true => orc(ecs, x, y),
        false => goblin(ecs, x, y),
    }
}

pub fn fill_room(ecs: &mut World, room: &IRect) {
    let mut monster_spawn_points = SmallVec::<[(i32, i32); MAX_MONSTERS as usize]>::new();
    let mut item_spawn_points = SmallVec::<[(i32, i32); MAX_ITEMS as usize]>::new();
    let mut rng = thread_rng();

    let num_monsters = rng.gen_range(1..=MAX_MONSTERS);
    let num_items = rng.gen_range(1..=MAX_ITEMS);

    for _ in 1..=num_monsters {
        let mut added = false;
        while !added {
            let x = rng.gen_range(room.x..=room.xx);
            let y = rng.gen_range(room.y..=room.yy);
            if !monster_spawn_points.contains(&(x, y)) { 
                monster_spawn_points.push((x, y));
                added = true;
            }
        }
    }

    for _ in 1..=num_items {
        let mut added = false;
        while !added {
            let x = rng.gen_range(room.x..=room.xx);
            let y = rng.gen_range(room.y..=room.yy);
            if !item_spawn_points.contains(&(x, y)) { 
                item_spawn_points.push((x, y));
                added = true;
            }
        }
    }

    for (x, y) in monster_spawn_points.iter() {
        random_monster(ecs, *x, *y);
    }

    for (x, y) in item_spawn_points.iter() {
        random_item(ecs, *x, *y);
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

fn random_item(ecs: &mut World, x: i32, y: i32) {
    match thread_rng().gen_range(1..=4) {
        1 => health_potion(ecs, x, y),
        2 => magic_missile_scroll(ecs, x, y),
        3 => fireball_scroll(ecs, x, y),
        4 => confusion_scroll(ecs, x, y),
        _ => unreachable!()
    }
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
