 use macroquad::prelude::IVec2;
use specs::{prelude::*, saveload::{SimpleMarker, SimpleMarkerAllocator}};

use crate::{
    comp::*, 
    util::{GameLog, colors::*, to_cp437 },
    gui::{self, MainMenuSelection, UIState}, 
    map::*, 
    player::*, 
    save_load, 
    spawner, 
    systems::*,
    screen::Screen,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    SaveGame,
    UI(UIState),
    Quit,
}

pub struct State {
    screen: Screen,
    ecs: World,
    ai_system: MonsterAI,
    item_use_system: ItemUseSystem,
    sorted_drawables: Vec<(Position, Renderable)>,
    dirty: bool,
}

impl State {
    pub fn new(screen: Screen) -> Self {
        let mut ecs = World::new();
        register_all_components(&mut ecs);

        ecs.register::<SimpleMarker<SerializeMe>>();
        ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

        ecs.insert(RunState::UI(UIState::MainMenu(MainMenuSelection::NewGame)));

        ecs.insert(GameLog::new().with("Hello world".to_owned()));

        let map = Map::new(80, 43);
        let (x, y) = map.rooms()[0].center();
        ecs.insert(IVec2::new(x, y));

        let player_entity = spawner::player(&mut ecs, x, y);
        ecs.insert(player_entity);

        for room in map.rooms().iter().skip(1) {
            spawner::fill_room(&mut ecs, room);
        }

        ecs.insert(map);

        Self { 
            screen, ecs, ai_system: MonsterAI::default(),
            item_use_system: ItemUseSystem::default(),
            sorted_drawables: vec![],
            dirty: true
        }
    }

    fn run_systems(&mut self) {
        VisibilitySystem.run_now(&self.ecs);
        self.ai_system.run_now(&self.ecs);
        MapIndexingSystem.run_now(&self.ecs);
        MeleeCombatSystem.run_now(&self.ecs);
        DamageSystem.run_now(&self.ecs);
        InventorySystem.run_now(&self.ecs);
        self.item_use_system.run_now(&self.ecs);
        ItemDropSystem.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn render(&mut self) {
        self.screen.clear();

        match *self.ecs.fetch::<RunState>() {
            RunState::UI(UIState::MainMenu(_)) | RunState::SaveGame => return,
            _ => (),
        };

        let map = self.ecs.fetch::<Map>();
        draw_map(&map, &mut self.screen);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        if self.dirty {
            self.sorted_drawables.clear();
            self.sorted_drawables.extend((&positions, &renderables)
                .join()
                .map(|(r, p)| (*r, *p))
            );
            self.sorted_drawables.sort_unstable_by_key(|(_, x)| x.order);
            self.dirty = false;
        }

        for (Position { x, y }, render) in self.sorted_drawables.iter().rev() {
            if map.tile_flags(*x, *y).visible {
                self.screen.draw_glyph(*x, *y, render.glyph, render.fg, render.bg);
            }
        }

        gui::draw_ui(&self.ecs, &mut self.screen);
    }

    pub fn tick(&mut self) -> bool {
        self.render();

        use RunState::*;
        let old_state = *self.ecs.fetch::<RunState>();
        let new_state = match old_state {
            PreRun => {
                self.run_systems();
                AwaitingInput
            },
            AwaitingInput => handle_input(&mut self.ecs),
            PlayerTurn => {
                self.run_systems();
                MonsterTurn
            },
            MonsterTurn => {
                self.run_systems();
                self.dirty = true;
                AwaitingInput
            }
            SaveGame => {
                save_load::save_game(&mut self.ecs);
                RunState::UI(UIState::MainMenu(MainMenuSelection::LoadGame))
            },
            UI(state) => gui::handle_state(state, &mut self.ecs, &mut self.screen),
            Quit => Quit,
        };
        *self.ecs.write_resource::<RunState>() = new_state;

        DamageSystem::delete_the_dead(&mut self.ecs);

        self.screen.flush();

        new_state != Quit
    }
}

fn draw_map(map: &Map, s: &mut Screen) {
    let bg = BLACK;
    let floor_fg = [0.5, 0.5, 0.5, 1.0];
    let wall_fg = [0.0, 1.0, 0.0, 1.0];

    let bounds = map.bounds();

    for y in 0..bounds.height() {
        for x in 0..bounds.width() {
            let tile_status = map.tile_flags(x, y);
            if !tile_status.revealed { continue; }

            let (mut fg, glyph) = match map.tile(x, y) {
                TileType::Floor => (floor_fg, to_cp437('.')),
                TileType::Wall => (wall_fg, to_cp437('#')),
            };
            
            if !tile_status.visible { fg = greyscale(fg); }
            s.draw_glyph(x, y, glyph, fg, bg);
        }
    }
}
