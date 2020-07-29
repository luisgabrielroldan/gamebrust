#[derive(Clone, Copy)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub tile: u8,
    pub bg_priority: bool,
    pub y_flip: bool,
    pub x_flip: bool,
    pub palette: u8,
}

impl Sprite {
    pub fn new(x: i32, y: i32, tile: u8, flags: u8) -> Self {
        Self {
            y: y as u8,
            x: x as u8,
            tile: tile,
            bg_priority: (flags >> 7) & 1 == 1,
            y_flip: (flags >> 6) & 1 == 1,
            x_flip: (flags >> 5) & 1 == 1,
            palette: (flags >> 4) & 1,
        }
    }
}
