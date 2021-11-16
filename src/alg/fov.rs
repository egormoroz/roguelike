use std::cmp::Ord;
use std::ops::Range;

use macroquad::math::IVec2;

use super::BaseMap;

pub fn compute_fov<M, F>(origin: IVec2, range: i32, map: &M, mut mark_visible: F)
where
    M: BaseMap,
    F: FnMut(IVec2)
{
    mark_visible(origin);

    for quadrant in Quadrant::all(origin) {
        let row = Row::new(1, Fraction::new(-1, 1), Fraction::new(1, 1));
        Scanner { quadrant, map, mark_visible: &mut mark_visible, range }
            .scan(row);
    }
}

struct Scanner<'a, M: BaseMap, F: FnMut(IVec2)> {
    quadrant: Quadrant, 
    map: &'a M, 
    mark_visible: &'a mut F,
    range: i32,
}

impl<'a, M: BaseMap, F: FnMut(IVec2)> Scanner<'a, M, F> {
    fn is_opaque(&self, tile: IVec2) -> bool {
        self.map.is_opaque(self.quadrant.transform(tile))
    }

    fn reveal(&mut self, tile: IVec2) {
        (self.mark_visible)(self.quadrant.transform(tile));
    }

    fn scan(&mut self, mut row: Row) {
        let mut prev_tile = None;
        for col in row.cols(self.range) {
            let tile = IVec2::new(col, row.depth);
            if self.is_opaque(tile) || row.is_symmetric(col) {
                self.reveal(tile);
            }

            if let Some(prev_tile) = prev_tile {
                if self.is_opaque(prev_tile) && !self.is_opaque(tile) {
                    row.start_slope = slope(col, row.depth);
                }
                if !self.is_opaque(prev_tile) && self.is_opaque(tile) {
                    self.scan(Row { end_slope: slope(col, row.depth), ..row});
                }
            }

            prev_tile = Some(tile);
        }

        if prev_tile.is_some() && !self.is_opaque(prev_tile.unwrap()) {
            self.scan(row.next());
        }
    }
}


#[derive(Clone, Copy)]
enum Cardinal { North, East, South, West }

struct Quadrant(Cardinal, IVec2);

impl Quadrant {
    fn all(origin: IVec2) -> [Quadrant; 4] {
        use Cardinal::*;
        [Quadrant(North, origin),
         Quadrant(East, origin),
         Quadrant(South, origin),
         Quadrant(West, origin)]
    }

    fn transform(&self, tile: IVec2) -> IVec2 {
        let (cardinal, origin) = (self.0, self.1);
        use Cardinal::*;

        origin + match cardinal {
            North => IVec2::new(tile.x, -tile.y),
            South => tile,
            East => IVec2::new(tile.y, tile.x),
            West => IVec2::new(-tile.y, tile.x),
        }
    }
}

#[derive(Clone, Copy)]
struct Fraction(i32, i32);

impl Fraction {
    fn new(num: i32, den: i32) -> Self {
        assert!(den > 0);
        Self(num, den)
    }

    fn round_ties_up(&self) -> i32 {
        let (a, b) = (self.0, self.1);
        if 2 * a + b >= 0 {
            (2 * a + b) / (2 * b)
        } else {
            ((2 * a + b).abs() + 2 * b - 1) / (-2 * b)
        }
    }

    fn round_ties_down(&self) -> i32 {
        let (a, b) = (self.0, self.1);
        if 2 * a - b >= 0 {
            (2 * a + b - 1) / (2 * b)
        } else {
            (2 * a - b) / (2 * b)
        }
    }

    fn mult(self, k: i32) -> Self {
        Self(self.0 * k, self.1)
    }
}

impl Eq for Fraction {}

impl PartialEq for Fraction {
    fn eq(&self, other: &Self) -> bool {
        self.0 * other.1 == other.0 * self.1
    }
}

impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 * other.1).cmp(&(other.0 * self.1))
    }
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}


#[derive(Clone, Copy)]
struct Row {
    depth: i32,
    start_slope: Fraction,
    end_slope: Fraction,
}

impl Row {
    fn new(depth: i32, start_slope: Fraction, end_slope: Fraction) -> Self {
        Self {
            depth,
            start_slope,
            end_slope
        }
    }

    fn cols(&self, r: i32) -> Range<i32> {
        let d = ((r * r - self.depth * self.depth) as f32).sqrt().floor() as i32;
        if d == 0 { return 0..0 }

        let start = self.start_slope.mult(self.depth).round_ties_up()
            .max(-d);
        let end = self.end_slope.mult(self.depth).round_ties_down()
            .min(d) + 1;
        start..end
    }

    fn next(&self) -> Self {
        Self {
            depth: self.depth + 1,
            ..*self
        }
    }

    fn is_symmetric(&self, col: i32) -> bool {
        let left = self.start_slope.mult(self.depth);
        let right = self.end_slope.mult(self.depth);
        let col = Fraction::new(col, 1);
        col >= left && col <= right
    }
}

fn slope(col: i32, depth: i32) -> Fraction {
    Fraction::new(2 * col - 1, 2 * depth)
}
