use macroquad::prelude::*;

use crate::util::to_cp437;

use super::util::{
    IRect,
    Glyph,
    Grid,
};


#[derive(Debug, Default, Clone, Copy)]
struct Cell {
    glyph: Glyph,
    bg: [f32; 4],
    fg: [f32; 4],
}

#[derive(Debug)]
pub struct Screen {
    buffer: Grid<Cell>,
    texture: Texture2D,
    glyph_size: Vec2,
    scaled_glyph_size: Vec2,
    cols: u8,
}

impl Screen {
    pub fn new(scr_width: i32, scr_height: i32, texture: Texture2D,
               cols: u8, rows: u8, scale: Vec2) -> Self 
    {

        let glyph_size = Vec2::new(texture.width() / cols as f32, texture.height() / rows as f32);
        Self {
            buffer: Grid::new(scr_width, scr_height, Cell::default()),
            scaled_glyph_size: glyph_size * scale,
            texture, cols, glyph_size,
        }
    }

    pub fn clear(&mut self) {
        for i in self.buffer.iter_mut() {
            *i = Cell::default();
        }
    }

    pub fn set_bg(&mut self, x: i32, y: i32, bg: [f32; 4]) {
        self.buffer.get_mut(x, y).bg = bg;
    }

    pub fn draw_glyph(&mut self, x: i32, y: i32, glyph: Glyph, fg: [f32; 4], bg: [f32; 4]) {
        *self.buffer.get_mut(x, y) = Cell { glyph, fg, bg };
    }

    pub fn draw_box(&mut self, bounds: IRect, fg: [f32; 4], bg: [f32; 4]) {
        for y in bounds.y..=bounds.yy {
            for x in bounds.x..=bounds.xx {
                *self.buffer.get_mut(x, y) = Cell::default();
            }
        }

        for x in bounds.x+1..bounds.xx {
            self.draw_glyph(x, bounds.y, to_cp437('─'), fg, bg);
            self.draw_glyph(x, bounds.yy, to_cp437('─'), fg, bg);
        }

        for y in bounds.y+1..bounds.yy {
            self.draw_glyph(bounds.x, y, to_cp437('│'), fg, bg);
            self.draw_glyph(bounds.xx, y, to_cp437('│'), fg, bg);
        }

        self.draw_glyph(bounds.x, bounds.y, to_cp437('┌'), fg, bg);
        self.draw_glyph(bounds.xx, bounds.y, to_cp437('┐'), fg, bg);
        self.draw_glyph(bounds.x, bounds.yy, to_cp437('└'), fg, bg);
        self.draw_glyph(bounds.xx, bounds.yy, to_cp437('┘'), fg, bg);
    }

    pub fn draw_text(&mut self, mut x: i32, y: i32, fg: [f32; 4], bg: [f32; 4], text: &str) {
        for ch in text.chars() {
            self.draw_glyph(x, y, to_cp437(ch), fg, bg);
            x += 1;
            if x >= self.buffer.width() { break; }
        }
    }

    pub fn draw_text_centered(&mut self, y: i32, fg: [f32; 4], bg: [f32; 4], text: &str) {
        let x = (self.buffer.width() - text.len() as i32) / 2;
        self.draw_text(x.max(0), y, fg, bg, text);
    }

    pub fn draw_bar_horizontal(&mut self, x: i32, y: i32, width: i32, value: i32, 
        max_value: i32, fg: [f32; 4], bg: [f32; 4]) 
    {
        let xx = x + width * value / max_value;
        for x in x..xx.min(self.buffer.width()) {
            self.draw_glyph(x, y, to_cp437('▓'), fg, bg);
        }
        for x in xx..(x + width).min(self.buffer.width()) {
            self.draw_glyph(x, y, to_cp437('░'), fg, bg);
        }
    }

    pub fn flush(&self) {
        clear_background(BLACK);
        for y in 0..self.buffer.height() {
            for x in 0..self.buffer.width() {
                let pos = Vec2::new(x as f32, y as f32) * self.scaled_glyph_size;
                let bg = self.buffer.get(x, y).bg;
                let bg = Color::new(bg[0], bg[1], bg[2], bg[3]);
                draw_rectangle(pos.x, pos.y, self.scaled_glyph_size.x, self.scaled_glyph_size.y, bg);
            }
        }

        for y in 0..self.buffer.height() {
            for x in 0..self.buffer.width() {
                let Cell { glyph, fg, bg: _ } = self.buffer.get(x, y);
                let r = self.get_rect(*glyph);
                let pos = Vec2::new(x as f32, y as f32) * self.scaled_glyph_size;
                let fg = Color::new(fg[0], fg[1], fg[2], fg[3]);

                draw_texture_ex(self.texture, pos.x, pos.y, fg, DrawTextureParams {
                    source: Some(r),
                    dest_size: Some(self.scaled_glyph_size),
                    ..DrawTextureParams::default()
                })
            }
        }
    }

    fn get_rect(&self, n: u8) -> Rect {
        let offset = Vec2::new((n % self.cols) as f32, (n / self.cols) as f32);
        Rect::new(0., 0., self.glyph_size.x, self.glyph_size.y)
            .offset(offset * self.glyph_size)
    }
}
