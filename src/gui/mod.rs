mod menu;
mod ui_state;
use std::io::Write;
use std::io::Cursor;

use macroquad::prelude::{IVec2, KeyCode, get_last_key_pressed};
use smallvec::SmallVec;
pub use menu::*;
pub use ui_state::*;


use specs::prelude::*;
use crate::{
    player::transform_movement_input,
    screen::Screen,
    comp::*, 
    util::{
        letter_to_option,
        GameLog, 
        IRect,
        colors::*,
        to_cp437,
        Glyph
    },
    map::Map,
};



pub enum ItemMenuResult { 
    Cancel, 
    NoResponse, 
    Selected,
}

pub fn draw_ui(ecs: &World, s: &mut Screen) {
    let depth = ecs.fetch::<Map>().depth();
    let players = ecs.read_storage::<Player>();
    let stats = ecs.read_storage::<CombatStats>();
    let hunger_clock = ecs.read_storage::<HungerClock>();
    let (stats, hc, _) = (&stats, &hunger_clock, &players).join().next().unwrap();

    s.draw_box(IRect::new(0, 43, 80, 7), WHITE, BLACK);

    s.draw_text(2, 43, YELLOW, BLACK, &format!("Depth: {}", depth));
    s.draw_text(12, 43, YELLOW, BLACK,
        &format!("HP: {} / {}", stats.hp, stats.max_hp));
    s.draw_bar_horizontal(28, 43, 51, stats.hp, 
        stats.max_hp, RED, BLACK);

    let log = ecs.fetch::<GameLog>();
    let mut y = 44;
    for entry in log.last_entries(5) {
        s.draw_text(2, y, WHITE, [0.0; 4], entry);
        y += 1;
    }

    use HungerState::*;
    let (text, fg) = match hc.state {
        WellFed => ("Well fed", GREEN),
        Normal => ("", [0.0; 4]),
        Hungry => ("Hungry", ORANGE),
        Starving => ("Starving", RED),
    };
    s.draw_text(71, 42, fg, BLACK, text);
}

pub fn show_inventory(ecs: &World, title: &str, s: &mut Screen) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = ecs.fetch::<Entity>();
    let named = ecs.read_storage::<Named>();
    let backpacked = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();

    let (result, selection) = match get_last_key_pressed() {
        Some(KeyCode::Escape) => (ItemMenuResult::Cancel, -1),
        Some(key) => (ItemMenuResult::Selected, letter_to_option(key)),
        None => (ItemMenuResult::NoResponse, -1),
    };

    let mut items: SmallVec<[(Entity, &str); 64]> = 
    (&entities, &backpacked, &named)
        .join()
        .filter(|(_, itm, _)| itm.owner == *player_entity)
        .map(|(e, _, name)| (e, name.0.as_str()))
        .collect();
    items.sort_unstable_by_key(|(_, x)| *x);
    let mut item_counts: SmallVec<[(usize, i32); 32]> = SmallVec::new();

    if !items.is_empty() { item_counts.push((0, 1)); }
    for i in 1..items.len() {
        if items[i - 1].1 == items[i].1 {
            item_counts.last_mut().unwrap().1 += 1;
            continue;
        }
        item_counts.push((i, 1));
    }

    let num_entries = item_counts.len() as i32;
    let mut y = (25 - (num_entries / 2)) as i32;
    s.draw_box(IRect::new(15, y - 1, 31, num_entries+2), WHITE, BLACK);
    s.draw_text(18, y-2, YELLOW, BLACK, title);
    s.draw_text(18, y + num_entries + 1, YELLOW, BLACK, "ESCAPE to cancel");

    let mut selected_itm = None;
    let mut buf = [0u8; 64];
    for (i, (idx, cnt)) in item_counts.iter().enumerate() {
        let (entity, name) = items[*idx];
        s.draw_glyph(17, y, to_cp437('['), WHITE, BLACK);
        s.draw_glyph(18, y, 97 + i as Glyph, WHITE, BLACK);
        s.draw_glyph(19, y, to_cp437(']'), WHITE, BLACK);

        let mut cursor = Cursor::new(&mut buf[..]);
        write!(cursor, "{}", name).unwrap();
        if *cnt > 1 {
            write!(cursor, " ({})", *cnt).unwrap();
        }
        let cursor = cursor.position() as usize;
        let text = std::str::from_utf8(&buf[..cursor]).unwrap();
        s.draw_text(21, y, WHITE, BLACK, text);

        y += 1;
        if selection == i as i32 {
            selected_itm = Some(entity);
        }
    }

    match result {
        ItemMenuResult::Selected if selected_itm.is_none() => 
            (ItemMenuResult::NoResponse, None),
        result => (result, selected_itm)
    }
}

pub fn show_examiner(ecs: &World, s: &mut Screen, mut pos: IVec2, range: Option<i32>) -> (ItemMenuResult, IVec2) {
    let player_entity = ecs.fetch::<Entity>();
    let plp = *ecs.fetch::<IVec2>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let viewshed = viewsheds.get(*player_entity).unwrap();
    let range = range.unwrap_or(viewshed.range);

    let d = plp - pos;
    if d.dot(d) > range * range {
        pos = plp;
    }


    let mut result = ItemMenuResult::NoResponse;

    if let Some(key) = get_last_key_pressed() {
        if let Some((dx, dy)) = transform_movement_input(key) {
            let dst = IVec2::new(dx, dy) + pos;
            let d = plp - dst;
            if d.dot(d) <= range *range && viewshed.can_see(dst.x, dst.y) {
                pos = dst;
            }
        }
        match key {
            KeyCode::Escape => result = ItemMenuResult::Cancel,
            KeyCode::Enter => result = ItemMenuResult::Selected,
            _ => (),
        }
    }

    for pt in viewshed.visible_tiles.iter() {
        let d = IVec2::new(pt.0, pt.1) - plp;
        if d.dot(d) <= range * range {
            s.set_bg(pt.0, pt.1, GRAY);
        }
    }

    s.set_bg(pos.x, pos.y, BLUE);

    (result, pos)
}

pub fn ranged_target(ecs: &World, s : &mut Screen, range: i32, pos: IVec2) -> (ItemMenuResult, IVec2) {
    s.draw_text(5, 0, YELLOW, BLACK, "Select target");
    show_examiner(ecs, s, pos, Some(range))
}

pub enum GameOverResult { Idle, Quit }

pub fn game_over(s : &mut Screen) -> GameOverResult {
    s.draw_text_centered(15, YELLOW, BLACK, "Your journey has ended!");
    s.draw_text_centered(17, WHITE, BLACK, "One day, we'll tell you all about how you did.");
    s.draw_text_centered(18, WHITE, BLACK, "That day, sadly, is not in this chapter..");
    s.draw_text_centered(20, MAGENTA, BLACK, "Press any key to return to the menu.");

    match get_last_key_pressed() {
        Some(_) => GameOverResult::Quit,
        None => GameOverResult::Idle
    }
}
