use macroquad::prelude::IVec2;
use specs::prelude::*;
use crate::{
    util::DjMap, 
    map::{Map, ViewMap, TileType},
    alg::BFS,
};

#[derive(Default)]
pub struct DjMapUpdateSystem {
    bfs: BFS<(i32, i32)>,
}

impl<'a> System<'a> for DjMapUpdateSystem {
    type SystemData = (
        ReadExpect<'a, IVec2>,
        ReadExpect<'a, Map>,
        WriteExpect<'a, DjMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (plp, map, mut dj_map) = data;
        if !dj_map.needs_updating(plp.x, plp.y) { return; }

        dj_map.reset(plp.x, plp.y, map.bounds());

        let adjacent = |g: &mut DjMap, (x, y): &(i32, i32)| 
            map.adjacent(*x, *y)
            .filter(|(x, y)| map.tile(*x, *y) == &TileType::Floor 
                && g.bounds().contains(*x, *y))
            .collect();
        let sources = [(plp.x, plp.y)];
        self.bfs.search(sources.into_iter(), &mut *dj_map, 
            |g, (x, y), c| g.set(*x, *y, c), 
            |g, (x, y)| g.get(*x, *y),
            adjacent,
        );
    }
}

