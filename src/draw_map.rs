use crate::{
    map::{TileType, TileFlags},
    screen::Screen,
    util::{IRect, to_cp437, colors::*, Glyph},
};

pub trait ViewMap {
    fn bounds(&self) -> IRect;

    fn tile_flags(&self, x: i32, y: i32) -> &TileFlags;
    fn tile(&self, x: i32, y: i32) -> &TileType;
}

pub fn draw_map<M: ViewMap>(map: &M, s: &mut Screen) {
    let bg = BLACK;
    let floor_fg = [0.0, 0.5, 0.5, 1.0];
    let wall_fg = [0.0, 1.0, 0.0, 1.0];
    let stairs_fg = VIOLET;

    let bounds = map.bounds();

    for y in 0..bounds.height() {
        for x in 0..bounds.width() {
            let tile_status = map.tile_flags(x, y);
            if !tile_status.revealed { continue; }

            let (mut fg, glyph) = match map.tile(x, y) {
                TileType::Floor => (floor_fg, to_cp437('.')),
                TileType::Wall => (wall_fg, wall_glyph(map, x, y)),
                TileType::DownStairs => (stairs_fg, to_cp437('>'))
            };
            let bg = match tile_status.bloodstained && tile_status.visible {
                true => [0.75, 0., 0., 1.],
                false => bg,
            };
            
            if !tile_status.visible { fg = greyscale(fg); }
            s.draw_glyph(x, y, glyph, fg, bg);
        }
    }
}

fn wall_glyph<M: ViewMap>(map: &M, x: i32, y: i32) -> Glyph {
    let bounds = map.bounds();
    
    let mut mask = 0;
    let test = |x, y| !bounds.contains(x, y) ||
        map.tile_flags(x, y).revealed && map.tile(x, y) == &TileType::Wall;

    if test(x, y - 1) { mask |= 1; }
    if test(x, y + 1) { mask |= 2; }
    if test(x - 1, y) { mask |= 4; }
    if test(x + 1, y) { mask |= 8; }

    [9, 186, 186, 186, 205, 188, 187, 185, 205, 200, 201,
     204, 205, 202, 203, 206][mask]
}

