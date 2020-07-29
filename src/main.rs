extern crate minifb;

use std::rc::Rc;
use std::cell::RefCell;
use minifb::{Key, ScaleMode, Window, WindowOptions};
use core::cartridge::Cartridge;
use core::io::joypad::JoypadKey;
use core::Display;
use core::System;

struct UI {
    frame: u32,
    w: usize,
    h: usize,
    pub window: Rc<RefCell<Window>>,
}

impl UI {
    pub fn new(w: usize, h: usize) -> (Self, Rc<RefCell<Window>>) {
        let mut window = Window::new(
            "GameBRust",
            w,
            h,
            WindowOptions {
                resize: true,
                scale_mode: ScaleMode::AspectRatioStretch,
                ..WindowOptions::default()
            },
            )
            .unwrap();

        // window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

        let window = Rc::new(RefCell::new(window));
        let rc = window.clone();

        (
            Self {
                w: w,
                h: h,
                window: window,
                frame: 0
            },
            rc,
            )
    }
}

impl Display for UI {
    fn update(&mut self, buffer: &Vec<u32>) {
        if self.frame % 2 > 0 {
            self.frame += 1;
        }

        self.window
            .borrow_mut()
            .update_with_buffer(buffer, self.w, self.h)
            .unwrap();

        self.frame = 0;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cartridge =
        // match Cartridge::from_path("roms/gb-test-roms/cpu_instrs/individual/08-misc instrs.gb") {
        match Cartridge::from_path("roms/galaga.gb") {
            Ok(cartridge) => cartridge,
            _ => panic!("Error!"),
        };

    let (display, window_rc) = UI::new(160, 144);

    let mut system = System::new(cartridge, Box::new(display), true);

    let keys = vec![
        (Key::Right, JoypadKey::Right),
        (Key::Up, JoypadKey::Up),
        (Key::Left, JoypadKey::Left),
        (Key::Down, JoypadKey::Down),
        (Key::Z, JoypadKey::A),
        (Key::X, JoypadKey::B),
        (Key::Space, JoypadKey::Select),
        (Key::Enter, JoypadKey::Start),
    ];

    loop {

        system.step();

        {
            let window = window_rc.borrow_mut();
            let joypad = system.get_joypad_adapter();

            if window.is_key_down(minifb::Key::Escape) {
                break;
            }

            for (k, s) in &keys {
                if window.is_key_down(*k) {
                    joypad.pressed(*s);
                } else {
                    joypad.released(*s);
                }
            }
        }
    }

    Ok(())
}
