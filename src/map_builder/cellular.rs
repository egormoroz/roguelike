use std::mem::take;
use rand::{thread_rng, Rng};
use simdnoise::{CellDistanceFunction, NoiseBuilder, CellReturnType};

use super::*;
use crate::{
    alg::BFS,
    util::adjacent
};


#[derive(Eq, PartialEq)]
enum Stage { 
    Init, 
    IterationFirst(i32), 
    IterationSecond(i32), 
    Finalize, 
    Done, 
}

pub struct CellularAutomata {
    tiles: Grid<TileType>,
    depth: i32,
    stage: Stage,
    plp: IVec2,
}

impl CellularAutomata {
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        Self {
            tiles: Grid::new(width, height, TileType::Wall),
            depth,
            stage: Stage::Init,
            plp: IVec2::new(0, 0),
        }
    }

    fn init(&mut self) {
        let bounds = IRect::new(1, 1, self.tiles.width() - 2, 
            self.tiles.height() - 2);

        let mut rng = thread_rng();
        for (x, y) in bounds.iter() {
            *self.tiles.get_mut(x, y) = if rng.gen_bool(0.45) {
                TileType::Wall
            } else {
                TileType::Floor
            };
        }
        self.stage = Stage::IterationFirst(0);
    }

    fn count_walls(&self, x: i32, y: i32, r: i32) -> i32 {
        let actual_bounds = IRect::new(0, 0, 
            self.tiles.width(), self.tiles.height());
        IRect { x: x - r, y: y - r, xx: x + r, yy: y + r }
            .intersection(&actual_bounds).unwrap()
            .iter()
            .filter(|(x, y)| self.tiles.get(*x, *y) == &TileType::Wall)
            .count() as i32
    }


    fn iteration_first(&mut self, n: i32) {
        let bounds = IRect::new(1, 1, self.tiles.width() - 2, 
            self.tiles.height() - 2);
        for (x, y) in bounds.iter() {
            let tile;
            if self.count_walls(x, y, 1) >= 5 || self.count_walls(x, y, 2) <= 2 {
                tile = TileType::Wall;
            } else {
                tile = TileType::Floor;
            }
            *self.tiles.get_mut(x, y) = tile;
        }

        self.stage = if n >= 3 {
            Stage::IterationSecond(0)
        } else {
            Stage::IterationFirst(n + 1)
        };
    }


    fn iteration_second(&mut self, n: i32) {
        let bounds = IRect::new(1, 1, self.tiles.width() - 2, 
            self.tiles.height() - 2);
        for (x, y) in bounds.iter() {
            let tile;
            if self.count_walls(x, y, 1) >= 5 {
                tile = TileType::Wall;
            } else {
                tile = TileType::Floor;
            }
            *self.tiles.get_mut(x, y) = tile;
        }

        self.stage = if n >= 2 {
            Stage::Finalize
        } else {
            Stage::IterationSecond(n + 1)
        };
    }

    fn finalize(&mut self) {
        let (w, h) = (self.tiles.width(), self.tiles.height());
        let bounds = IRect::new(0, 0, w, h);
        let mut bfs = BFS::default();
        let mut g = Grid::new(w, h, -1);

        let my_adjacent = |_: &mut Grid<_>, (x, y): &(i32, i32)| 
            adjacent(*x, *y)
            .filter(|(x, y)| bounds.contains(*x, *y))
            .collect();

        let (plx, ply) = bfs.search_until([(w / 2, h / 2)].into_iter(), 
            &mut g, 
            |g, (x, y), c| *g.get_mut(*x, *y) = c, 
            |g, (x, y)| *g.get(*x, *y), 
            my_adjacent,
            |_, (x, y)| self.tiles.get(*x, *y) == &TileType::Floor
        ).unwrap();
        self.plp = IVec2::new(plx, ply);

        let adjacent = |_: &mut Grid<_>, (x, y): &(i32, i32)| 
            adjacent(*x, *y)
            .filter(|(x, y)| bounds.contains(*x, *y)
                && self.tiles.get(*x, *y) == &TileType::Floor)
            .collect();

        for i in g.iter_mut() { *i = -1; }
        bfs.search([(plx, ply)].into_iter(), 
            &mut g, 
            |g, (x, y), c| *g.get_mut(*x, *y) = c, 
            |g, (x, y)| *g.get(*x, *y), 
            adjacent
        );

        for (x, y) in bounds.iter() {
            if g.get(x, y) == &-1 {
                *self.tiles.get_mut(x, y) = TileType::Wall;
            }
        }

        let (exit_x, exit_y, _) = bounds
            .iter()
            .map(|(x, y)| (x, y, *g.get(x, y)))
            .max_by_key(|(_, _, d)| *d).unwrap();
        *self.tiles.get_mut(exit_x, exit_y) = TileType::DownStairs;

        let cnt = self.tiles.iter()
            .filter(|&tt| tt == &TileType::Floor)
            .count();
        let r = cnt as f32 / (w * h) as f32;
        if (0.4..=0.6).contains(&r) {
            self.stage = Stage::Done;
        } else {
            self.stage = Stage::Init;
        }
    }

    fn spawn_in(&self, ecs: &mut World, spawner: &mut Spawner, area: &[usize]) {
        let w = self.tiles.width();
        const MAX_DEPTH1_SPAWNS: i32 = 4;
        spawner.set_depth(self.depth);

        let mut rng = thread_rng();
        let max_spawns = (MAX_DEPTH1_SPAWNS + self.depth).min(area.len() as i32);

        let mut needed = rng.gen_range(1..=max_spawns) as u32;
        let mut left = area.len() as u32;
        for i in area {
            if rng.gen_range(0.0..1.0) < needed as f32 / left as f32 {
                let i = *i as i32;
                let (x, y) = (i % w, i / w);
                if self.tiles.get(x, y) == &TileType::Floor {
                    spawner.spawn(ecs, x, y);
                    needed -= 1;
                }
            }
            left -= 1;
        }
    }
}

impl MapBuilder for CellularAutomata {
    fn progress(&mut self) -> bool {
        match self.stage {
            Stage::Init => self.init(),
            Stage::IterationFirst(n) => self.iteration_first(n),
            Stage::IterationSecond(n) => self.iteration_second(n),
            Stage::Finalize => self.finalize(),
            Stage::Done => (),
        }

        self.stage == Stage::Done
    }

    fn spawn(&self, ecs: &mut World, spawner: &mut Spawner) {
        let (w, h) = (self.tiles.width(), self.tiles.height());
        let (noise, _, _) = NoiseBuilder::cellular_2d(w as usize, h as usize)
            .with_seed(1337)
            .with_distance_function(CellDistanceFunction::Manhattan)
            .with_return_type(CellReturnType::CellValue)
            .with_freq(0.08)
            .generate();
        let mut indices = (0..(w * h) as usize).collect::<Vec<_>>();
        indices.sort_unstable_by(|idx1, idx2| 
            noise[*idx1].partial_cmp(&noise[*idx2]).unwrap());

        let (mut start, mut start_idx) = (0, indices[0]);
        for (i, idx) in indices.iter().skip(1).enumerate() {
            if (noise[*idx] - noise[start_idx]).abs() > f32::EPSILON {
                println!("[{}; {}]", start, i);
                self.spawn_in(ecs, spawner, &indices[start..i + 1]);
                start = i;
                start_idx = *idx;
            }
        }
        self.spawn_in(ecs, spawner, &indices[start..]);
    }

    fn player_pos(&self) -> IVec2 { self.plp }

    fn intermediate(&self) -> IntermediateMap {
        IntermediateMap { tiles: &self.tiles }
    }

    fn build(&mut self) -> Map { Map::from_grid(take(&mut self.tiles), self.depth) }
}

