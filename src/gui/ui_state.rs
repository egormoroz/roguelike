use specs::prelude::*;
use macroquad::prelude::IVec2;
use smallvec::{smallvec, SmallVec};
use std::io::Write;

use super::*;
use crate::{
    map::Map,
    screen::Screen,
    state::RunState,
    save_load,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIState {
    UseItem,
    DropItem,
    Examine(IVec2),
    Target { range: i32, item: Entity, pos: IVec2 },
    MainMenu(MainMenuSelection),
}

pub fn handle_state(state: UIState, ecs: &mut World, s: &mut Screen) -> RunState {
    use UIState::*;
    match state {
        UseItem => match show_inventory(ecs, "Inventory", s) {
            (ItemMenuResult::Cancel, _) => RunState::AwaitingInput,
            (ItemMenuResult::Selected, Some(item)) => {
                let ranged = ecs.read_storage::<Ranged>();
                let plp = *ecs.fetch::<IVec2>();
                if let Some(ranged) = ranged.get(item) {
                    RunState::UI(Target { item, range: ranged.range, pos: plp })
                } else {
                    ecs.write_storage::<WantsToUseItem>()
                        .insert(*ecs.fetch::<Entity>(), WantsToUseItem { item, target: UseTarget::User })
                        .expect("unable to insert intent");
                    RunState::PlayerTurn
                }
            }
            _ => RunState::UI(UseItem)
        },
        DropItem => match show_inventory(ecs, "Drop which item?", s) {
            (ItemMenuResult::Cancel, _) => RunState::AwaitingInput,
            (ItemMenuResult::Selected, Some(item)) => {
                ecs.write_storage::<WantsToDropItem>()
                    .insert(*ecs.fetch::<Entity>(), WantsToDropItem { item })
                    .expect("unable to insert intent");

                RunState::PlayerTurn
            }
            _ => RunState::UI(DropItem)
        },
        Examine(initial) => match show_examiner(ecs, s, initial, None) {
            (ItemMenuResult::Selected, epos) => {
                let mut log = ecs.fetch_mut::<GameLog>();
                let map = ecs.fetch::<Map>();
                let names = ecs.read_storage::<Named>();
                let mut items: SmallVec<[&str; 8]> = smallvec![];

                for e in map.tile_content(epos.x, epos.y) {
                    if let Some(name) = names.get(*e) {
                        items.push(&name.0);
                    }
                }

                let mut entry = log.new_entry();
                match items.len() {
                    0 => write!(entry, "This is a {:?}.", map.tile(epos.x, epos.y)).unwrap(),
                    1 => write!(entry, "There is {}.", items[0]).unwrap(),
                    n => {

                        write!(entry, "There are {} entities: {}", n, items[0]).unwrap();
                        for name in &items[1..] {
                            write!(entry, ", {}", name).unwrap();
                        }
                        write!(entry, ".").unwrap();
                    }
                };

                RunState::AwaitingInput                    
            },
            (ItemMenuResult::NoResponse, epos) => RunState::UI(Examine(epos)),
            _ => RunState::AwaitingInput,
        }
        Target { range, item, pos } =>  {
            match ranged_target(ecs, s, range, pos) {
                (ItemMenuResult::Selected, pos) => {
                    let player = *ecs.fetch::<Entity>();
                    let (x, y) = (pos.x, pos.y);
                    ecs.write_storage::<WantsToUseItem>()
                        .insert(player, WantsToUseItem { item, target: UseTarget::Point((x, y)) })
                        .expect("failed to insert intent");
                    RunState::PlayerTurn                        
                },
                (ItemMenuResult::NoResponse, pos) => RunState::UI(Target { range, item, pos }),
                _ => RunState::AwaitingInput,
            }
        }
        MainMenu(current) => match main_menu(ecs, s, current) {
                MainMenuResult::Idle(selection) => RunState::UI(MainMenu(selection)),
                MainMenuResult::Selected(selection) => match selection {
                    MainMenuSelection::NewGame => RunState::PreRun,
                    MainMenuSelection::Quit=> RunState::Quit,
                    MainMenuSelection::LoadGame => {
                        save_load::load_game(ecs);
                        // save_load::delete_save();
                        RunState::PreRun
                    }
                }
            }
    }
}
