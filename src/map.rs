use specs::Entity;
use serde::{Serialize, Deserialize};
use macroquad::prelude::IVec2;
use smallvec::SmallVec;

use super::{
    util::{
        IRect,
        Grid,
    },
    alg::BaseMap,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TileFlags {
    pub revealed: bool,
    pub visible: bool,
    pub blocked: bool,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Map {
    tiles: Grid<TileType>,
    tile_flags: Grid<TileFlags>,
    rooms: Vec<IRect>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    tile_content: Grid<Vec<Entity>>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        let mut inst = Self {
            tiles: Grid::new(width, height, TileType::Wall),
            tile_flags: Grid::new(width, height, TileFlags::default()),
            rooms: vec![],
            tile_content: Grid::new(width, height, vec![]),
        };

        let rooms = vec![
            IRect::new(5, 5, 10, 10),
            IRect::new(3, 20, 12, 15),
            IRect::new(20, 25, 20, 12),
            IRect::new(18, 8, 20, 15),
            IRect::new(50, 2, 10, 10),
            IRect::new(48, 20, 20, 10),
            IRect::new(60, 35, 15, 6),
        ];

        for r in &rooms {
            inst.create_room(r);
        }

        for (r1, r2) in rooms.iter().zip(rooms.iter().skip(1)) {
            let (c1, c2) = (r1.center(), r2.center());
            inst.create_corridor(c1.0, c1.1, c2.0, c2.1);
        }

        inst.rooms = rooms;
        inst.populate_blocked();
        inst
    }

    pub fn rooms(&self) -> &[IRect] {
        &self.rooms
    }

    pub fn realloc_content_index(&mut self) {
        let (width, height) = (self.tiles.width(), self.tiles.height());
        self.tile_content.resize(width, height, vec![]);
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn populate_blocked(&mut self) {
        let it_flags = self.tile_flags.iter_mut();
        for (flags, tile) in it_flags.zip(self.tiles.iter()) {
            flags.blocked = *tile == TileType::Wall;
        }
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: TileType) {
        *self.tiles.get_mut(x, y) = tile;
    }

    pub fn tile(&self, x: i32, y: i32) -> TileType {
        *self.tiles.get(x, y)
    }

    pub fn create_room(&mut self, r: &IRect) {
        for y in r.y..=r.yy {
            for x in r.x..=r.xx {
                self.set_tile(x, y, TileType::Floor);
            }
        }
    }

    pub fn create_corridor(&mut self, x: i32, y: i32, xx: i32, yy: i32) {
        for x in x.min(xx)..=x.max(xx) {
            self.set_tile(x, y, TileType::Floor);
        }

        for y in y.min(yy)..=y.max(yy) {
            self.set_tile(x.max(xx), y, TileType::Floor);
        }
    }

    pub fn bounds(&self) -> IRect {
        IRect::new(0, 0, self.tiles.width(), self.tiles.height())
    }

    pub fn tile_flags(&self, x: i32, y: i32) -> TileFlags {
        *self.tile_flags.get(x, y)
    }

    pub fn tile_flags_mut(&mut self, x: i32, y: i32) -> &mut TileFlags {
        self.tile_flags.get_mut(x, y)
    }

    pub fn reset_visible_tiles(&mut self) {
        for st in self.tile_flags.iter_mut() {
            st.visible = false;
        }
    }

    pub fn tile_content(&self, x: i32, y: i32) -> &[Entity] {
        &self.tile_content.get(x, y)
    }

    pub fn tile_content_mut(&mut self, x: i32, y: i32) -> &mut Vec<Entity> {
        self.tile_content.get_mut(x, y)
    }
    
    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        self.bounds().contains(x, y) && !self.tile_flags(x, y).blocked
    }
}

impl BaseMap for Map {
    fn size(&self) -> IVec2 {
        IVec2::new(self.tiles.width(), self.tiles.height())
    }

    fn is_opaque(&self, pos: IVec2) -> bool {
        self.tile(pos.x, pos.y) == TileType::Wall
    }

    fn successors(&self, pos: IVec2) -> SmallVec<[(IVec2, f32); 8]> {
        use std::f32::consts::SQRT_2;
        [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)]
            .into_iter()
            .map(|(dx, dy)| (pos + IVec2::new(dx, dy), if dx * dy == 0 { 1.0 } else { SQRT_2 }))
            .filter(|(pos, _)| self.is_exit_valid(pos.x, pos.y))
            .collect()
    }
}
