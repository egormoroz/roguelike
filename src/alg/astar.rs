use macroquad::math::IVec2;
use smallvec::SmallVec;
use super::BaseMap;

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap}
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Step {
    successor: IVec2,
    cost: f32,
}

impl Eq for Step {}

impl Ord for Step {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.partial_cmp(&self.cost).unwrap()
    }
}

impl PartialOrd for Step {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Default)]
pub struct AStarPath {
    to_see: BinaryHeap<Step>,
    seen: HashMap<IVec2, (IVec2, f32)>,
    path: Vec<(IVec2, f32)>,
}

impl AStarPath {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compute_generic<H, S>(&mut self, from: IVec2, mut to: IVec2, 
        heuristic: &mut H, successors: &mut S) 
    where
        H: FnMut(IVec2, IVec2) -> f32,
        S: FnMut(IVec2) -> SmallVec<[(IVec2, f32); 8]>,
    {
        self.to_see.clear();
        self.seen.clear();
        self.path.clear();

        self.to_see.push(Step {
            cost: 0.,
            successor: from,
        });
        self.seen.insert(from, (from, 0.));

        while let Some(Step { successor, cost }) = self.to_see.pop() {
            if successor == to { break; }
            let c = self.cost_to(successor);
            if cost > c + heuristic(successor, to) { continue; }

            for (n, d) in successors(successor) {
                if c + d < self.cost_to(n) {
                    self.seen.insert(n, (successor, c + d));
                    self.to_see.push(Step { 
                        cost: c + d + heuristic(n, to),
                        successor: n,
                    });
                }
            }
        }

        if let Some((_, cost)) = self.seen.get(&to) {
            self.path.push((to, *cost));
            while let Some(x) = self.seen.get(&to) {
                self.path.push(*x);
                to = x.0;
                if to == from { break; }
            }
        }
    }

    pub fn compute<M: BaseMap>(&mut self, map: &M, from: IVec2, mut to: IVec2) {
        self.to_see.clear();
        self.seen.clear();
        self.path.clear();

        self.to_see.push(Step {
            cost: 0.,
            successor: from,
        });
        self.seen.insert(from, (from, 0.));

        while let Some(Step { successor, cost }) = self.to_see.pop() {
            if successor == to { break; }
            let c = self.cost_to(successor);
            if cost > c + map.distance(successor, to) { continue; }

            for (n, d) in map.successors(successor) {
                if c + d < self.cost_to(n) {
                    self.seen.insert(n, (successor, c + d));
                    self.to_see.push(Step { 
                        cost: c + d + map.distance(n, to),
                        successor: n,
                    });
                }
            }
        }

        if let Some((_, cost)) = self.seen.get(&to) {
            self.path.push((to, *cost));
            while let Some(x) = self.seen.get(&to) {
                self.path.push(*x);
                to = x.0;
                if to == from { break; }
            }
        }
    }

    fn cost_to(&self, to: IVec2) -> f32 {
        self.seen.get(&to).map(|(_, c)| *c).unwrap_or(f32::INFINITY)
    }

    ///The path is reversed
    pub fn result(&self) -> &[(IVec2, f32)] {
        &self.path[..]
    }
}

