use std::collections::VecDeque;
use std::mem::take;
use rand::{thread_rng, Rng, rngs::ThreadRng};
use super::*;

#[derive(PartialEq, Eq)]
enum Stage { Partition, Collect, Done }

pub struct BSP {
    tiles: Grid<TileType>,
    rooms: Vec<IRect>,
    split_queue: VecDeque<IRect>,
    depth: i32,
    rng: ThreadRng,
    stage: Stage,
}

impl BSP {
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        Self {
            tiles: Grid::new(width, height, TileType::Floor),
            rooms: vec![],
            split_queue: [IRect::new(0, 0, width, height)].into(),
            depth,
            rng: thread_rng(),
            stage: Stage::Partition,
        }
    }

    fn create_walls(&mut self, r: &IRect) {
        for x in r.x..=r.xx {
            *self.tiles.get_mut(x, r.y) = TileType::Wall;
            *self.tiles.get_mut(x, r.yy) = TileType::Wall;
        }

        for y in r.y..=r.yy {
            *self.tiles.get_mut(r.x, y) = TileType::Wall;
            *self.tiles.get_mut(r.xx, y) = TileType::Wall;
        }
    }

    fn partition(&mut self) {
        if let Some(r) = self.split_queue.pop_front() {
            self.create_walls(&r);
            if let Some((r1, r2)) = split(r, &mut self.rng) {
                self.split_queue.push_back(r1);
                self.split_queue.push_back(r2);
            } else {
                self.split_queue.push_front(r);
                self.stage = Stage::Collect;
            }
        } else {
            self.stage = Stage::Collect;
        }
    }

    fn collect(&mut self) {
        if let Some(r) = self.split_queue.pop_front() {
            self.create_walls(&r);
            self.rooms.push(r);
        } else {
            for r in self.rooms.iter() {
                *self.tiles.get_mut(r.xx, (r.y + r.yy) / 2) = TileType::Floor;
                *self.tiles.get_mut((r.x + r.xx) / 2, r.yy) = TileType::Floor;
            }
            self.create_walls(&IRect::new(0, 0, self.tiles.width(), self.tiles.height()));

            self.stage = Stage::Done;
        }
    }
}

impl MapBuilder for BSP {
    fn progress(&mut self) -> bool {
        match self.stage {
            Stage::Partition => self.partition(),
            Stage::Collect => self.collect(),
            Stage::Done => (),
        };
        self.stage == Stage::Done
    }

    fn spawn(&self, ecs: &mut World, spawner: &mut Spawner) {
        const MAX_DEPTH1_SPAWNS: i32 = 4;
        spawner.set_depth(self.depth);

        let mut rng = thread_rng();
        let num_spawns = rng.gen_range(1..=MAX_DEPTH1_SPAWNS + self.depth);
        let mut spawn_points = Vec::with_capacity(num_spawns as usize);

        for room in self.rooms.iter().skip(1) {
            for _ in 1..=num_spawns {
                loop {
                    let x = rng.gen_range(room.x + 1..room.xx);
                    let y = rng.gen_range(room.y + 1..room.yy);
                    if !spawn_points.contains(&(x, y)) { 
                        spawn_points.push((x, y));
                        break;
                    }
                }
            }
        }

        for (x, y) in spawn_points {
            spawner.spawn(ecs, x, y);
        }
    }

    fn player_pos(&self) -> IVec2 { 
        let (x, y) = self.rooms[0].center();
        IVec2::new(x, y)
    }

    fn intermediate(&self) -> IntermediateMap {
        IntermediateMap { tiles: &self.tiles }
    }

    fn build(&mut self) -> Map { Map::from_grid(take(&mut self.tiles), self.depth) }
}

fn split<R: Rng>(r: IRect, rng: &mut R) -> Option<(IRect, IRect)> {
    const MIN_SIZE: i32 = 6;
    let roll = rng.gen_bool(sigmoid(r.width() as f64 / r.height() as f64 - 1.));
    let (w, h) = ((r.width() - 1) / 2, (r.height() - 1) / 2);

    //0.3..0.7
    let min_w = MIN_SIZE.max(r.width() * 3 / 10);
    let min_h = MIN_SIZE.max(r.height() * 3 / 10);
    let max_w = (r.width() - MIN_SIZE).min(r.width() * 7 / 10);
    let max_h = (r.height() - MIN_SIZE).min(r.height() * 7 / 10);

    if w >= MIN_SIZE && (h < MIN_SIZE || roll) {
        let w = rng.gen_range(min_w..=max_w);
        return Some((IRect::new(r.x, r.y, w, r.height()),
            IRect::new(r.x + w - 1, r.y, r.width() - w + 1, r.height())));
    }

    if h >= MIN_SIZE && (w < MIN_SIZE || !roll) {
        let h = rng.gen_range(min_h..=max_h);
        return Some((IRect::new(r.x, r.y, r.width(), h),
            IRect::new(r.x, r.y + h - 1, r.width(), r.height() - h + 1)));
    }

    None
}

fn sigmoid(x: f64) -> f64 {
    1. / (1. + f64::exp(-x))
}
