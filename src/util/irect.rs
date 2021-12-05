use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct IRect {
    pub x: i32,
    pub y: i32,
    pub xx: i32,
    pub yy: i32,
}

impl IRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x, y, xx: x + w - 1, yy: y + h - 1
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, i32)> {
        let (x, y, xx, yy) = (self.x, self.y, self.xx, self.yy);
        (y..=yy).flat_map(move |y| (x..=xx).map(move |x| (x, y)))
    }

    pub fn width(&self) -> i32 { self.xx - self.x + 1 }
    pub fn height(&self) -> i32 { self.yy - self.y + 1 }

    pub fn overlaps(&self, other: &IRect) -> bool {
        self.x <= other.xx && self.xx >= other.x 
            && self.y <= other.yy && self.yy >= other.y
    }

    pub fn intersection(&self, other: &IRect) -> Option<IRect> {
        if self.overlaps(other) {
            Some(IRect {
                x: self.x.max(other.x),
                y: self.y.max(other.y),
                xx: self.xx.min(other.xx),
                yy: self.yy.min(other.yy),
            })
        } else {
            None
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.x <= x && self.y <= y 
            && x <= self.xx && y <= self.yy
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x + self.xx) / 2, (self.y + self.yy) / 2)
    }
}

