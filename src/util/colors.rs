pub const LIGHTGRAY: [f32; 4] = [0.78, 0.78, 0.78, 1.00];
pub const GRAY: [f32; 4] = [0.51, 0.51, 0.51, 1.00];
pub const DARKGRAY: [f32; 4] = [0.31, 0.31, 0.31, 1.00];
pub const YELLOW: [f32; 4] = [0.99, 0.98, 0.00, 1.00];
pub const GOLD: [f32; 4] = [1.00, 0.80, 0.00, 1.00];
pub const ORANGE: [f32; 4] = [1.00, 0.63, 0.00, 1.00];
pub const PINK: [f32; 4] = [1.00, 0.43, 0.76, 1.00];
pub const RED: [f32; 4] = [0.90, 0.16, 0.22, 1.00];
pub const MAROON: [f32; 4] = [0.75, 0.13, 0.22, 1.00];
pub const GREEN: [f32; 4] = [0.00, 0.89, 0.19, 1.00];
pub const LIME: [f32; 4] = [0.00, 0.62, 0.18, 1.00];
pub const DARKGREEN: [f32; 4] = [0.00, 0.46, 0.17, 1.00];
pub const SKYBLUE: [f32; 4] = [0.40, 0.75, 1.00, 1.00];
pub const BLUE: [f32; 4] = [0.00, 0.47, 0.95, 1.00];
pub const DARKBLUE: [f32; 4] = [0.00, 0.32, 0.67, 1.00];
pub const PURPLE: [f32; 4] = [0.78, 0.48, 1.00, 1.00];
pub const VIOLET: [f32; 4] = [0.53, 0.24, 0.75, 1.00];
pub const DARKPURPLE: [f32; 4] = [0.44, 0.12, 0.49, 1.00];
pub const BEIGE: [f32; 4] = [0.83, 0.69, 0.51, 1.00];
pub const BROWN: [f32; 4] = [0.50, 0.42, 0.31, 1.00];
pub const DARKBROWN: [f32; 4] = [0.30, 0.25, 0.18, 1.00];
pub const WHITE: [f32; 4] = [1.00, 1.00, 1.00, 1.00];
pub const BLACK: [f32; 4] = [0.00, 0.00, 0.00, 1.00];
pub const BLANK: [f32; 4] = [0.00, 0.00, 0.00, 0.00];
pub const MAGENTA: [f32; 4] = [1.00, 0.00, 1.00, 1.00];

pub const CYAN: [f32; 4] = [0.00, 0.68, 0.93, 1.00];

pub fn greyscale(c: [f32; 4]) -> [f32; 4] {
    let linear = c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722;
    [linear, linear, linear, c[3]]
}
