use std::io::Write;
use macroquad::prelude::*;
use specs::prelude::*;
use crate::{
    comp::*, 
    util::GameLog, 
    map::{Map, TileType}, 
    state::RunState,
    gui::UIState,
};

pub fn try_move_player(dx: i32, dy: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();

    let (entity, _, pos, viewshed) = (&entities, &players, &mut positions, &mut viewsheds).join().next().unwrap();

    let (dst_x, dst_y) = (pos.x + dx, pos.y + dy);

    if !map.tile_flags(dst_x, dst_y).blocked {
        pos.x = dst_x;
        pos.y = dst_y;
        viewshed.dirty = true;
        *ecs.write_resource::<IVec2>() = IVec2::new(dst_x, dst_y);
        RunState::PlayerTurn
    } else {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

        for potential_target in map.tile_content(dst_x, dst_y) {
            if let Some(_target) = combat_stats.get(*potential_target) {
                wants_to_melee.insert(entity, WantsToMelee { target: *potential_target }).unwrap();
                return RunState::PlayerTurn;
            }
        }

        RunState::AwaitingInput
    }
}


pub fn transform_movement_input(key: KeyCode) -> Option<(i32, i32)> {
    match key {
        KeyCode::Left | KeyCode::H => Some((-1, 0)),
        KeyCode::Right | KeyCode::L => Some((1, 0)),
        KeyCode::Up | KeyCode::K => Some((0, -1)),
        KeyCode::Down | KeyCode::J => Some((0, 1)),
        KeyCode::Y => Some((-1, -1)),
        KeyCode::U => Some((1, -1)),
        KeyCode::B => Some((-1, 1)),
        KeyCode::N => Some((1, 1)),
        _ => None,
    }
}

pub fn handle_input(ecs: &mut World) -> RunState {
    let plp = *ecs.fetch::<IVec2>();
    if let Some(key) = get_last_key_pressed() {
        if let Some((dx, dy)) = transform_movement_input(key) {
            return try_move_player(dx, dy, ecs);
        }
        match key {
            //Inventory, items
            KeyCode::I => RunState::UI(UIState::UseItem),
            KeyCode::D => RunState::UI(UIState::DropItem),
            KeyCode::G => get_item(ecs),

            //Misc
            KeyCode::X => RunState::UI(UIState::Examine(plp)),
            KeyCode::Space => RunState::PlayerTurn,
            KeyCode::Escape => RunState::SaveGame,
            KeyCode::Period => try_go_deeper(ecs, plp),
            _ => RunState::AwaitingInput,
        }
    } else {
        RunState::AwaitingInput
    }
}

fn get_item(ecs: &mut World) -> RunState {
    let player_pos = ecs.fetch::<IVec2>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut log = ecs.write_resource::<GameLog>();

    let mut target_item = None;
    for (entity, pos, _) in (&entities, &positions, &items).join() {
        if pos.x == player_pos.x && pos.y == player_pos.y {
            target_item = Some(entity);
            break;
        }
    }

    if let Some(item) = target_item {
        ecs.write_storage::<WantsToPickupItem>()
            .insert(*player_entity, WantsToPickupItem { item })
            .expect("unable to insert WantToPickupItem");
        RunState::PlayerTurn
    } else {
        write!(log.new_entry(), "There is nothing here to pick up").unwrap();
        RunState::AwaitingInput
    }

}

fn try_go_deeper(ecs: &World, plp: IVec2) -> RunState {
    if let TileType::DownStairs = ecs.fetch::<Map>().tile(plp.x, plp.y) {
        RunState::NextLevel
    } else {
        write!(ecs.fetch_mut::<GameLog>().new_entry(),
            "There is no way down from here.").unwrap();
        RunState::AwaitingInput
    }
}
