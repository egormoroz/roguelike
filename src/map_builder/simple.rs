use rand::{thread_rng, Rng};
use super::*;
use crate::util::{Grid, IRect};

pub struct SimpleBuilder {
    tiles: Grid<TileType>,
    rooms: Vec<IRect>,
    plp: IVec2,
    depth: i32,
}

impl SimpleBuilder {
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        Self {
            tiles: Grid::new(width, height, TileType::Wall),
            depth,
            plp: IVec2::new(0, 0),
            rooms: vec![],
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
            *self.tiles.get_mut(x.max(xx), y) = TileType::Floor;
        }
    }
}

impl MapBuilder for SimpleBuilder {
    fn generate(&mut self) {
        let rooms = vec![
            IRect::new(5, 5, 10, 10),
            IRect::new(3, 20, 12, 15),
            IRect::new(20, 25, 20, 12),
            IRect::new(18, 8, 20, 15),
            IRect::new(50, 2, 10, 10),
            IRect::new(48, 20, 20, 10),
            IRect::new(60, 35, 15, 6)
        ];

        let (x, y) = rooms[0].center();
        self.plp = IVec2::new(x, y);

        for r in &rooms {
            self.create_room(r);
        }

        for (r1, r2) in rooms.iter().zip(rooms.iter().skip(1)) {
            let ((x, y), (xx, yy)) = (r1.center(), r2.center());
            self.create_corridor(x, y, xx, yy);
        }

        let (x, y) = rooms.last().unwrap().center();
        *self.tiles.get_mut(x, y) = TileType::DownStairs;
        self.rooms = rooms;
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

    fn player_pos(&self) -> IVec2 { self.plp }
    fn build(self) -> Map { Map::from_grid(self.tiles, self.depth) }
}
