use crate::util::{
    Grid,
    IRect,
};


//TODO: Update lazily!!!
pub struct DjMap {
    map: Grid<i32>,
    bounds: IRect,
    plp: Option<(i32, i32)>,
}

const OFFSETS: [(i32, i32); 8] = [
    (-1, -1), (0, -1), (1, -1),
    (-1, 0), (1, 0),
    (-1, 1), (0, 1), (1, 1),
];

impl DjMap {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            bounds: IRect::new(0, 0, width, height),
            map: Grid::new(width, height, -1),
            plp: None,
        }
    }

    pub fn max_width(&self) -> i32 { self.map.width() }
    pub fn max_height(&self) -> i32 { self.map.height() }

    pub fn bounds(&self) -> IRect { self.bounds }

    pub fn needs_updating(&self, x: i32, y: i32) -> bool {
        self.plp != Some((x, y))
    }

    pub fn reset(&mut self, x: i32, y: i32, real_bounds: IRect) {
        self.plp = Some((x, y));
        let left = x - self.max_width() / 2;
        let top = y - self.max_width() / 2;
        self.bounds = IRect::new(left, top, self.max_width(), self.max_height())
            .intersection(&real_bounds).unwrap();
        for x in self.map.iter_mut() {
            *x = -1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &i32> {
        self.map.iter().filter(|&x| *x >= 0)
    }

    pub fn get(&self, x: i32, y: i32) -> i32 {
        debug_assert!(self.bounds.contains(x, y));
        *self.map.get(x - self.bounds.x, y - self.bounds.y)
    }

    pub fn set(&mut self, x: i32, y: i32, v: i32) {
        debug_assert!(self.bounds.contains(x, y));
        *self.map.get_mut(x - self.bounds.x, y - self.bounds.y) = v;
    }

    pub fn adjacent(&self, x: i32, y: i32) -> impl Iterator<Item = (i32, i32, i32)>  + '_ {
        OFFSETS
            .iter()
            .map(move |(dx, dy)| (x + dx, y + dy))
            .filter(|(x, y)| self.bounds.contains(*x, *y) 
                && self.get(*x, *y) >= 0)
            .map(|(x, y)| (x, y, self.get(x, y)))
    }
}

