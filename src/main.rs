extern crate macroquad;
extern crate phf;
extern crate smallvec;
extern crate rand;
extern crate specs;
extern crate specs_derive;
extern crate serde;
extern crate simdnoise;

pub mod alg;
pub mod util;
pub mod map;
pub mod comp;
pub mod spawner;
pub mod save_load;
pub mod gui;
pub mod screen;
pub mod systems;
pub mod state;
pub mod player;
pub mod random_table;
pub mod map_builder;
pub mod draw_map;

use macroquad::prelude::*;

use screen::Screen;
use state::State;

fn window_conf() -> Conf {
    Conf {
        window_title: "Roguelike".to_owned(),
        window_width: 80 * 16,
        window_height: 50 * 16,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let texture = load_texture("atlas.png").await.unwrap();
    let screen = Screen::new(80, 50, texture, 16, 16, Vec2::new(1., 1.));

    let mut state = State::new(screen);

    while state.tick() {
        next_frame().await
    }
}

