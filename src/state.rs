use std::io::Write;
use macroquad::prelude::get_frame_time;
use specs::{prelude::*, saveload::SimpleMarkerAllocator};

use crate::{
    comp::*, 
    gui::{self, MainMenuSelection, UIState, GameOverResult}, 
    map::*, 
    map_builder::*, 
    player::*, 
    save_load, 
    screen::Screen, 
    spawner::{self, Spawner}, 
    systems::*, 
    util::*,
    draw_map::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapGenFinish {
    NextLevel,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    NewGame,
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    SaveGame,
    UI(UIState),
    Quit,
    NextLevel,
    GameOver,
    MagicMapReveal { row: i32 },
    GeneratingMap(MapGenFinish),
}

pub struct State {
    screen: Screen,
    spawner: Spawner,
    ecs: World,
    dj_system: DjMapUpdateSystem,
    ai_system: MonsterAI,
    item_use_system: ItemUseSystem,
    particle_system: ParticleSystem,
    sorted_drawables: Vec<(Position, Renderable)>,
    map_builder: Option<Box<dyn MapBuilder>>,
    mapgen_timer: f32,
}

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 43;

impl State {
    pub fn new(screen: Screen) -> Self {
        let mut ecs = World::new();
        register_all_components(&mut ecs);

        ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
        ecs.insert(RunState::UI(UIState::MainMenu(MainMenuSelection::NewGame)));
        ecs.insert(GameLog::default());
        ecs.insert(ParticleBuilder::default());
        ecs.insert(DeltaTime::default());
        ecs.insert(DjMap::new(40, 40));

        Self { 
            screen, ecs, 
            dj_system: DjMapUpdateSystem::default(),
            ai_system: MonsterAI::default(),
            item_use_system: ItemUseSystem::default(),
            particle_system: ParticleSystem::default(),
            sorted_drawables: vec![],
            spawner: Spawner::new(1),
            map_builder: None,
            mapgen_timer: 0.,
        }
    }

    fn run_systems(&mut self) {
        self.dj_system.run_now(&self.ecs);
        VisibilitySystem.run_now(&self.ecs);
        self.ai_system.run_now(&self.ecs);
        MapIndexingSystem.run_now(&self.ecs);
        TriggerSystem.run_now(&self.ecs);
        MeleeCombatSystem.run_now(&self.ecs);
        DamageSystem.run_now(&self.ecs);
        InventorySystem.run_now(&self.ecs);
        self.item_use_system.run_now(&self.ecs);
        ItemDropSystem.run_now(&self.ecs);
        ParticleSpawnSystem.run_now(&self.ecs);
        HungerSystem.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn render(&mut self) {
        self.screen.clear();
        self.particle_system.update(&mut self.ecs);

        let state = *self.ecs.fetch::<RunState>();
        match state {
            RunState::UI(UIState::MainMenu(_)) | RunState::SaveGame | RunState::GameOver 
                | RunState::NewGame => return,
            _ => (),
        };

        if let RunState::GeneratingMap(_) = state {
            let map = self.map_builder.as_ref().unwrap().intermediate();
            draw_map(&map, &mut self.screen);
            return;
        }

        let map = self.ecs.fetch::<Map>();
        draw_map(&*map, &mut self.screen);
        let dm = self.ecs.fetch::<DjMap>();
        self.screen.draw_djmap(&*dm);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let hidden = self.ecs.read_storage::<Hidden>();

        self.sorted_drawables.clear();
        self.sorted_drawables.extend((&positions, &renderables, !&hidden)
            .join()
            .map(|(r, p, _)| (*r, *p))
        );
        self.sorted_drawables.sort_unstable_by_key(|(_, x)| x.order);

        for (Position { x, y }, render) in self.sorted_drawables.iter().rev() {
            if map.tile_flags(*x, *y).visible {
                self.screen.draw_glyph(*x, *y, render.glyph, render.fg, render.bg);
            }
        }

        gui::draw_ui(&self.ecs, &mut self.screen);
    }

    pub fn tick(&mut self) -> bool {
        let dt = get_frame_time() * 1000.;
        self.ecs.write_resource::<DeltaTime>().0 = dt;
        self.render();

        use RunState::*;
        let old_state = *self.ecs.fetch::<RunState>();
        let new_state = match old_state {
            NewGame => self.reset(),
            PreRun => {
                self.run_systems();
                AwaitingInput
            },
            AwaitingInput => handle_input(&mut self.ecs),
            PlayerTurn => {
                self.run_systems();
                match *self.ecs.fetch::<RunState>() {
                    mmr @ MagicMapReveal { row: _ } => mmr,
                    _ => MonsterTurn,
                }
            },
            MonsterTurn => {
                self.run_systems();
                AwaitingInput
            }
            SaveGame => {
                save_load::save_game(&mut self.ecs);
                RunState::UI(UIState::MainMenu(MainMenuSelection::LoadGame))
            },
            UI(state) => gui::handle_state(state, &mut self.ecs, &mut self.screen),
            Quit => Quit,
            NextLevel => self.goto_next_level(),
            GameOver => match gui::game_over(&mut self.screen) {
                GameOverResult::Idle => GameOver,
                GameOverResult::Quit => RunState::UI(UIState::MainMenu(MainMenuSelection::NewGame))
            },
            MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                let bounds = map.bounds();
                for x in 0..bounds.width() {
                    map.tile_flags_mut(x, row).revealed = true;
                }

                if row + 1 >= bounds.height() {
                    MonsterTurn
                } else {
                    MagicMapReveal { row: row + 1 }
                }
            },
            GeneratingMap(finish) if self.mapgen_timer < 0. => {
                self.mapgen_timer = 100.;
                if self.map_builder.as_mut().unwrap().progress() {
                    self.gen_world_finish();
                    match finish {
                        MapGenFinish::NextLevel => self.goto_next_level_finish(),
                        MapGenFinish::Reset => (),
                    };
                    PreRun
                } else {
                    GeneratingMap(finish)
                }
            }
            state @ GeneratingMap(_) => {
                self.mapgen_timer -= dt;
                state
            }
        };
        *self.ecs.write_resource::<RunState>() = new_state;

        DamageSystem::delete_the_dead(&mut self.ecs);

        self.screen.flush();

        new_state != Quit
    }

    fn goto_next_level(&mut self) -> RunState {
        let mut to_delete = vec![];
        let player_entity = *self.ecs.fetch::<Entity>();
        {
            let entities = self.ecs.entities();
            let in_backpack = self.ecs.read_storage::<InBackpack>();
            let equipped = self.ecs.read_storage::<Equipped>();

            for e in entities.join() {
                if e == player_entity { continue }

                if let Some(bp) = in_backpack.get(e) {
                    if bp.owner == player_entity { continue }
                }
                if let Some(Equipped { slot: _, owner }) = equipped.get(e) {
                    if *owner == player_entity { continue }
                }
                to_delete.push(e);
            }
        }
        self.ecs.delete_entities(&to_delete).expect("failed to delete entities");
        let depth = self.ecs.fetch::<Map>().depth() + 1;
        self.gen_world(depth);
        RunState::GeneratingMap(MapGenFinish::NextLevel)
    }

    fn goto_next_level_finish(&mut self) {
        let player_entity = *self.ecs.fetch::<Entity>();
        let mut stats = self.ecs.write_storage::<CombatStats>();
        let stats = stats.get_mut(player_entity)
            .expect("player doesn't have CombatStats??");
        stats.hp = stats.max_hp.min(stats.hp + stats.max_hp / 2);

        write!(self.ecs.fetch_mut::<GameLog>().new_entry(),
            "You descend to the next level, and take a moment to heal.").unwrap();
    }

    fn reset(&mut self) -> RunState {
        self.ecs.delete_all();
        {
            let mut log = self.ecs.fetch_mut::<GameLog>();
            log.clear();
            write!(log.new_entry(), "Hello world").unwrap();
        }

        self.gen_world(1);
        RunState::GeneratingMap(MapGenFinish::Reset)
    }

    fn gen_world(&mut self, depth: i32) {
        self.map_builder = Some(Box::new(BSPGen::new(MAP_WIDTH, MAP_HEIGHT, depth)));
    }

    fn gen_world_finish(&mut self) {
        let mut builder = self.map_builder.take().unwrap();
        builder.spawn(&mut self.ecs, &mut self.spawner);
        let plp = builder.player_pos();
        let map = builder.build();
        self.ecs.insert(plp);
        self.ecs.insert(map);

        if !self.ecs.read_storage::<Player>().is_empty() {
            let player_entity = *self.ecs.fetch::<Entity>();
            *self.ecs.write_storage::<Position>()
                .get_mut(player_entity).unwrap() = Position { x: plp.x, y: plp.y };
            self.ecs.write_storage::<Viewshed>()
                .get_mut(player_entity).unwrap().dirty = true;
        } else {
            let player_entity = spawner::player(&mut self.ecs, plp.x, plp.y);
            self.ecs.insert(player_entity);
        }
    }
}

