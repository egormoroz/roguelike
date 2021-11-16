use std::io::Write;
use macroquad::prelude::{IVec2, get_frame_time};
use specs::{prelude::*, saveload::SimpleMarkerAllocator};

use crate::{
    comp::*, 
    util::{GameLog, colors::*, to_cp437, Glyph, DeltaTime },
    gui::{self, MainMenuSelection, UIState, GameOverResult}, 
    map::*, 
    player::*, 
    save_load, 
    spawner::{self, RoomSpawner}, 
    systems::*,
    screen::Screen,
};


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
    MagicMapReveal { row: i32 }
}

pub struct State {
    screen: Screen,
    spawner: RoomSpawner,
    ecs: World,
    ai_system: MonsterAI,
    item_use_system: ItemUseSystem,
    particle_system: ParticleSystem,
    sorted_drawables: Vec<(Position, Renderable)>,
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

        Self { 
            screen, ecs, ai_system: MonsterAI::default(),
            item_use_system: ItemUseSystem::default(),
            particle_system: ParticleSystem::default(),
            sorted_drawables: vec![],
            spawner: RoomSpawner::new(1),
        }
    }

    fn run_systems(&mut self) {
        VisibilitySystem.run_now(&self.ecs);
        self.ai_system.run_now(&self.ecs);
        MapIndexingSystem.run_now(&self.ecs);
        MeleeCombatSystem.run_now(&self.ecs);
        InventorySystem.run_now(&self.ecs);
        self.item_use_system.run_now(&self.ecs);
        ItemDropSystem.run_now(&self.ecs);
        ParticleSpawnSystem.run_now(&self.ecs);
        HungerSystem.run_now(&self.ecs);
        DamageSystem.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn render(&mut self) {
        self.screen.clear();
        self.particle_system.update(&mut self.ecs);

        match *self.ecs.fetch::<RunState>() {
            RunState::UI(UIState::MainMenu(_)) | RunState::SaveGame 
                | RunState::GameOver | RunState::NewGame => return,
            _ => (),
        };

        let map = self.ecs.fetch::<Map>();
        draw_map(&map, &mut self.screen);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        self.sorted_drawables.clear();
        self.sorted_drawables.extend((&positions, &renderables)
            .join()
            .map(|(r, p)| (*r, *p))
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
        self.ecs.write_resource::<DeltaTime>().0 = get_frame_time() * 1000.;
        self.render();

        use RunState::*;
        let old_state = *self.ecs.fetch::<RunState>();
        let new_state = match old_state {
            NewGame => {
                self.reset();
                PreRun
            }
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
            NextLevel => {
                self.goto_next_level();
                PreRun
            },
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
            }
        };
        *self.ecs.write_resource::<RunState>() = new_state;

        DamageSystem::delete_the_dead(&mut self.ecs);

        self.screen.flush();

        new_state != Quit
    }

    fn goto_next_level(&mut self) {
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
        let new_map = Map::new(MAP_WIDTH, MAP_HEIGHT, depth);
        self.spawner.set_depth(depth);
        for room in new_map.rooms().iter().skip(1) {
            self.spawner.spawn(&mut self.ecs, room);
        }

        let (plx, ply) = new_map.rooms()[0].center();
        self.ecs.insert(IVec2::new(plx, ply));
        self.ecs.write_storage::<Position>()
            .insert(player_entity, Position { x: plx, y: ply })
            .expect("failed to insert player position");
        self.ecs.write_storage::<Viewshed>()
            .get_mut(player_entity)
            .expect("player doesn't have Viewshed??")
            .dirty = true;

        
        let mut stats = self.ecs.write_storage::<CombatStats>();
        let stats = stats.get_mut(player_entity)
            .expect("player doesn't have CombatStats??");
        stats.hp = stats.max_hp.min(stats.hp + stats.max_hp / 2);

        *self.ecs.fetch_mut::<Map>() = new_map;

        write!(self.ecs.fetch_mut::<GameLog>().new_entry(),
            "You descend to the next level, and take a moment to heal.").unwrap();
    }

    fn reset(&mut self) {
        self.ecs.delete_all();
        {
            let mut log = self.ecs.fetch_mut::<GameLog>();
            log.clear();
            write!(log.new_entry(), "Hello world").unwrap();
        }

        let map = Map::new(MAP_WIDTH, MAP_HEIGHT, 1);
        let (x, y) = map.rooms()[0].center();
        self.ecs.insert(IVec2::new(x, y));

        let player_entity = spawner::player(&mut self.ecs, x, y);
        self.ecs.insert(player_entity);

        self.spawner.set_depth(1);
        for room in map.rooms().iter().skip(1) {
            self.spawner.spawn(&mut self.ecs, room);
        }
        self.ecs.insert(map);
    }
}

fn draw_map(map: &Map, s: &mut Screen) {
    let bg = BLACK;
    let floor_fg = [0.0, 0.5, 0.5, 1.0];
    let wall_fg = [0.0, 1.0, 0.0, 1.0];
    let stairs_fg = VIOLET;

    let bounds = map.bounds();

    for y in 0..bounds.height() {
        for x in 0..bounds.width() {
            let tile_status = map.tile_flags(x, y);
            if !tile_status.revealed { continue; }

            let (mut fg, glyph) = match map.tile(x, y) {
                TileType::Floor => (floor_fg, to_cp437('.')),
                TileType::Wall => (wall_fg, wall_glyph(map, x, y)),
                TileType::DownStairs => (stairs_fg, to_cp437('>'))
            };
            let bg = match tile_status.bloodstained && tile_status.visible {
                true => [0.75, 0., 0., 1.],
                false => bg,
            };
            
            if !tile_status.visible { fg = greyscale(fg); }
            s.draw_glyph(x, y, glyph, fg, bg);
        }
    }
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> Glyph {
    let bounds = map.bounds();
    if x < 1 || y < 1 || x >= bounds.xx || y >= bounds.yy { return 35; }
    
    let mut mask = 0;
    let test = |x, y| map.tile_flags(x, y).revealed 
        && map.tile(x, y) == TileType::Wall;

    if test(x, y - 1) { mask |= 1; }
    if test(x, y + 1) { mask |= 2; }
    if test(x - 1, y) { mask |= 4; }
    if test(x + 1, y) { mask |= 8; }

    [9, 186, 186, 186, 205, 188, 187, 185, 205, 200, 201,
     204, 205, 202, 203, 206][mask]
}
