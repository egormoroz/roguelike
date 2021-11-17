use rand::{thread_rng, Rng};
use super::*;
use crate::util::{Grid, IRect};

pub struct SimpleBuilder {
    tiles: Grid<TileType>,
    rooms: Vec<IRect>,
    plp: IVec2,
    depth: i32,

    room_idx: i32,
}

impl SimpleBuilder {
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        Self {
            tiles: Grid::new(width, height, TileType::Wall),
            depth,
            plp: IVec2::new(0, 0),
            rooms: vec![],
            room_idx: 0,
        }
    }
    

    fn create_room(&mut self, r: &IRect) {
        for y in r.y..=r.yy {
            for x in r.x..=r.xx {
                *self.tiles.get_mut(x, y) = TileType::Floor;
            }
        }
    }

    fn create_corridor(&mut self, x: i32, y: i32, xx: i32, yy: i32) {
        for x in x.min(xx)..=x.max(xx) {
            *self.tiles.get_mut(x, y) = TileType::Floor;
        }

        for y in y.min(yy)..=y.max(yy) {
            *self.tiles.get_mut(xx, y) = TileType::Floor;
        }
    }
}


impl MapBuilder for SimpleBuilder {
    fn progress(&mut self) -> bool {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = thread_rng();
        let w = rng.gen_range(MIN_SIZE..=MAX_SIZE);
        let h = rng.gen_range(MIN_SIZE..=MAX_SIZE);
        let x = rng.gen_range(2..self.tiles.width() - 2 - w);
        let y = rng.gen_range(2..self.tiles.height() - 2 - h);

        let new_room = IRect::new(x, y, w, h);
        if self.rooms.iter().find(|&r| r.overlaps(&new_room)).is_none() {
            self.create_room(&new_room);
            if let Some(prev) = self.rooms.last() {
                let ((x, y), (xx, yy)) = (prev.center(), new_room.center());
                if rng.gen() {
                    self.create_corridor(x, y, xx, yy);
                } else {
                    self.create_corridor(xx, yy, x, y);
                }
            }
            self.rooms.push(new_room);
        }


        if self.room_idx >= MAX_ROOMS {
            let (x, y) = self.rooms.last().unwrap().center();
            *self.tiles.get_mut(x, y) = TileType::DownStairs;
            let (x, y) = self.rooms[0].center();
            self.plp = IVec2::new(x, y);
            true
        } else {
            self.room_idx += 1;
            false
        }
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
                    let x = rng.gen_range(room.x..=room.xx);
                    let y = rng.gen_range(room.y..=room.yy);
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

    fn intermediate(&self) -> IntermediateMap { 
        IntermediateMap { tiles: &self.tiles }
    }

    fn player_pos(&self) -> IVec2 { self.plp }
    fn build(&mut self) -> Map { Map::from_grid(std::mem::take(&mut self.tiles), self.depth) }
}
