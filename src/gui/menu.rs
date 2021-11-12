use macroquad::prelude::{KeyCode, get_last_key_pressed};
use specs::prelude::*;

use crate::{
    screen::Screen,
    util::colors::*,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit, }

#[derive(Debug, Clone, Copy)]
pub enum MainMenuResult {
    Idle(MainMenuSelection),
    Selected(MainMenuSelection)
}

pub fn main_menu(_ecs: &World, s: &mut Screen, selection: MainMenuSelection) -> MainMenuResult {
    s.draw_text_centered(15, YELLOW, BLACK, "My Roguelike");
    use MainMenuSelection::*;

    let options = [(NewGame, "Begin new game"), (LoadGame, "Load game"), (Quit, "Quit")];
    let opt_by_id = |idx| options[(0.max(idx) as usize).min(options.len() - 1)].0;
    let mut opt_id = 0;

    for (i, (opt, opt_name)) in options.iter().enumerate() {
        let fg = if *opt == selection { 
            opt_id = i as i32; MAGENTA 
        } else { 
            WHITE 
        };
        s.draw_text_centered(24 + i as i32, fg, BLACK, opt_name);
    }

    let key = get_last_key_pressed();
    if key.is_none() { return MainMenuResult::Idle(selection); }

    match key.unwrap() {
        KeyCode::Down | KeyCode::J => MainMenuResult::Idle(opt_by_id(opt_id + 1)),
        KeyCode::Up | KeyCode::K => MainMenuResult::Idle(opt_by_id(opt_id - 1)),
        KeyCode::Enter => MainMenuResult::Selected(selection),
        KeyCode::Escape => MainMenuResult::Selected(Quit),
        _ => MainMenuResult::Idle(selection),
    }
}
