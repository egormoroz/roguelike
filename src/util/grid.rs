use serde::{Deserialize, Serialize};
use std::slice::{Iter, IterMut};


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Grid<T: Clone> {
    data: Vec<T>,
    width: i32,
    height: i32,
}

impl<T: Clone> Grid<T> {
    pub fn new(width: i32, height: i32, value: T) -> Self {
        debug_assert!(width >= 0 && height >= 0);
        Self {
            data: vec![value; (width * height) as usize],
            width, height
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.data.iter_mut()
    }
    
    pub fn resize(&mut self, width: i32, height: i32, value: T) {
        debug_assert!(width >= 0 && height >= 0);
        self.data.resize((width * height) as usize, value);
        self.width = width;
        self.height = height;
    }

    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }

    pub fn get(&self, x: i32, y: i32) -> &T { 
        &self.data[self.xy_idx(x, y)]
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> &mut T {
        let idx = self.xy_idx(x, y);
        &mut self.data[idx]
    }

    fn xy_idx(&self, x: i32, y: i32) -> usize {
        debug_assert!(x >= 0 && y >= 0);
        (y * self.width + x) as usize
    }
}
