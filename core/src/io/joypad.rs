use crate::io;

#[derive(PartialEq, Clone, Copy)]
pub enum JoypadKey {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    A = 4,
    B = 5,
    Start = 6,
    Select = 7,
}

enum Mode {
    Buttons,
    Directions,
    Invalid,
}

pub struct Joypad {
    mode: Mode,
    intfs: u8,
    keys: [bool; 8],
}

pub trait JoypadAdapter {
    fn pressed(&mut self, key: JoypadKey);
    fn released(&mut self, key: JoypadKey);
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            intfs: 0,
            mode: Mode::Buttons,
            keys: [false; 8],
        }
    }

    pub fn step(&mut self) -> u8 {
        let intfs = self.intfs;
        self.intfs = 0;
        intfs
    }

    pub fn write(&mut self, v: u8) {
        self.mode = if v & (1 << 4) == 0 {
            Mode::Directions
        } else if v & (1 << 5) == 0 {
            Mode::Buttons
        } else {
            Mode::Invalid
        };
    }

    pub fn read(&self) -> u8 {
        match self.mode {
            Mode::Directions => self.get_directions() | (1 << 5),
            Mode::Buttons => self.get_buttons() | (1 << 4),
            _ => 0xFF,
        }
    }

    fn get_directions(&self) -> u8 {
        use JoypadKey::*;
        !(((if self.keys[Right as usize] { 1 } else { 0 })
            | (if self.keys[Left as usize] { 1 << 1 } else { 0 })
            | (if self.keys[Up as usize] { 1 << 2 } else { 0 })
            | (if self.keys[Down as usize] { 1 << 3 } else { 0 })) as u8) & 0x0F
    }

    fn get_buttons(&self) -> u8 {
        use JoypadKey::*;
        !(
            ((if self.keys[A as usize] { 1 } else { 0 }) | 
             (if self.keys[B as usize] { 1 << 1 } else { 0 }) | 
             (if self.keys[Select as usize] { 1 << 2 } else { 0 }) | 
             (if self.keys[Start as usize] { 1 << 3 } else { 0 })) as u8
         ) & 0x0F
    }

}

impl JoypadAdapter for Joypad {
    fn pressed(&mut self, key: JoypadKey) {
        if self.keys[key as usize] == false {
            self.keys[key as usize] = true;
            self.intfs = io::intf_raise(0, io::Flag::Joypad);
        }
    }

    fn released(&mut self, key: JoypadKey) {
        if self.keys[key as usize] == true {
            self.keys[key as usize] = false;
            self.intfs = io::intf_raise(0, io::Flag::Joypad);
        }
    }
}
