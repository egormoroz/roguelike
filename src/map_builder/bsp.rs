use macroquad::prelude::IVec2;
use smallvec::SmallVec;
use std::{collections::VecDeque, mem::take};
use rand::{thread_rng, Rng, rngs::ThreadRng};
use crate::alg::AStarPath;
use super::*;

#[derive(PartialEq, Eq)]
enum Stage { Partition, TrimRooms, Corridors, Done }

pub struct BSPGen {
    tiles: Grid<TileType>,
    split_queue: VecDeque<IRect>,
    room_idx: usize,
    depth: i32,
    rng: ThreadRng,
    stage: Stage,
    pf_cache: AStarPath,
}

impl BSPGen {
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        Self {
            tiles: Grid::new(width, height, TileType::Floor),
            split_queue: [IRect::new(0, 0, width, height)].into(),
            room_idx: 0,
            depth,
            rng: thread_rng(),
            stage: Stage::Partition,
            pf_cache: AStarPath::new(),
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
        let r = self.split_queue.pop_front().unwrap();
        if let Some((r1, r2)) = split(r, &mut self.rng) {
            self.create_walls(&r1);
            self.create_walls(&r2);
            self.split_queue.push_back(r1);
            self.split_queue.push_back(r2);
        } else {
            self.stage = Stage::TrimRooms;
            self.split_queue.push_front(r);
            self.room_idx = 0;
        }
    }

    fn trim_rooms(&mut self) {
        let mut rect = self.split_queue[self.room_idx];
        for y in rect.y..=rect.yy {
            for x in rect.x..=rect.xx {
                *self.tiles.get_mut(x, y) = TileType::Wall;
            }
        }

        trim_rect(&mut rect, &mut self.rng);
        self.split_queue[self.room_idx] = rect;

        for y in rect.y + 1..rect.yy {
            for x in rect.x + 1..rect.xx {
                *self.tiles.get_mut(x, y) = TileType::Floor;
            }
        }

        self.room_idx += 1;
        if self.room_idx >= self.split_queue.len() {
            self.stage = Stage::Corridors;
            self.room_idx = 1;
        }
    }

    fn corridors(&mut self) {
        let (fx, fy) = self.split_queue[self.room_idx - 1].center();
        let (tx, ty) = self.split_queue[self.room_idx].center();
        let (from, to) = (IVec2::new(fx, fy), IVec2::new(tx, ty));
        let bounds = IRect::new(1, 1, self.tiles.width() - 1, self.tiles.height() - 1);

        let mut successors = |n: IVec2| -> SmallVec<[(IVec2, f32); 8]>{
            [(1, 0), (0, 1), (-1, 0), (0, -1)]
                .into_iter()
                .map(|(dx, dy)| (n.x + dx, n.y + dy))
                .filter(|(x, y)| bounds.contains(*x, *y))
                .map(|(x, y)| (IVec2::new(x, y), if self.tiles.get(x, y) == &TileType::Floor { 1. } else { 5. }))
                .collect()
        };
        let mut heuristic = |a: IVec2, b: IVec2| (b.x - a.x + b.y - a.y).abs() as f32;
        self.pf_cache.compute_generic(from, to, &mut heuristic, &mut successors);

        for (n, _) in self.pf_cache.result() {
            *self.tiles.get_mut(n.x, n.y) = TileType::Floor;
        }

        self.room_idx += 1;
        if self.room_idx >= self.split_queue.len() {
            self.stage = Stage::Done;
        }
    }
}

impl MapBuilder for BSPGen {
    fn progress(&mut self) -> bool {
        match self.stage {
            Stage::Partition => self.partition(),
            Stage::TrimRooms => self.trim_rooms(),
            Stage::Corridors => self.corridors(),
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

        for room in self.split_queue.iter().skip(1) {
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
        let (x, y) = self.split_queue[0].center();
        IVec2::new(x, y)
    }

    fn intermediate(&self) -> IntermediateMap {
        IntermediateMap { tiles: &self.tiles }
    }

    fn build(&mut self) -> Map { Map::from_grid(take(&mut self.tiles), self.depth) }
}

const MIN_SIZE: i32 = 9;

fn split<R: Rng>(r: IRect, rng: &mut R) -> Option<(IRect, IRect)> {
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

fn trim_rect<R: Rng>(r: &mut IRect, rng: &mut R) {
    let min_w = MIN_SIZE.max(r.width() * 5 / 10);
    let min_h = MIN_SIZE.max(r.height() * 5 / 10);
    let max_w = MIN_SIZE.max(r.width() * 9 / 10);
    let max_h = MIN_SIZE.max(r.height() * 9 / 10);

    let w = rng.gen_range(min_w..=max_w);
    let h = rng.gen_range(min_h..=max_h);

    let (wspace, hspace) = (r.width() - w, r.height() - h);
    let (offx, offy) = (rng.gen_range(0..=wspace), rng.gen_range(0..=hspace));

    r.x += offx;
    r.y += offy;
    r.xx -= wspace - offx;
    r.yy -= hspace - offy;
}
