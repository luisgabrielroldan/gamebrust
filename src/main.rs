extern crate minifb;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::rc::Rc;
use std::cell::RefCell;
use minifb::{Key, ScaleMode, Window, WindowOptions};
use core::cartridge::Cartridge;
use core::io::joypad::JoypadKey;
use core::Display;
use core::System;
use std::time::Instant;
use std::path::Path;

struct UI {
    frame_tx: Sender<Vec<u32>>
}

impl UI {
    pub fn new(sender: Sender<Vec<u32>>) -> Self {
        Self {
            frame_tx: sender,
        }
    }
}

impl Display for UI {
    fn update(&mut self, buffer: &Vec<u32>) {
        self.frame_tx.send(buffer.to_vec()).unwrap();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<_> = std::env::args().collect();

    if argv.len() < 2 {
        println!("Usage: {} <rom-file>", argv[0]);
        return Ok(());
    }

    let rompath = String::from(&argv[1]);

    let (frame_tx, frame_rx) = mpsc::channel();
    let (input_tx, input_rx) = mpsc::channel();

    let mut window = Window::new(
        "GameBRust",
        160,
        144,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
        )
        .unwrap();

    let cpu_thread = thread::spawn(move || {
        let rompath = Path::new(&rompath);
        let cartridge =
            match Cartridge::from_path(rompath) {
                Ok(cartridge) => cartridge,
                _ => panic!("Error!"),
            };

        let display = UI::new(frame_tx);
        let mut system = System::new(cartridge, Box::new(display), false);

        let mut last_step = Instant::now();

        let mut ticks = 0;
        loop {
            last_step = Instant::now();
            let batch_ticks = (16 as f64 * (4_194_304 as f64 / 1000_f64)) as u32;

            while ticks < batch_ticks {
                ticks += system.step();
            }

            ticks -= batch_ticks;

            while (last_step.elapsed().as_millis() as u32) < 16 {
                let joypad = system.get_joypad_adapter();

                match input_rx.try_recv() {
                    Ok((key, true)) => { joypad.pressed(key); }
                    Ok((key, false)) => { joypad.released(key); }
                    _ => { } 
                };
            }
        }
    });

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

    let mut last_frame: Instant = Instant::now();

    while window.is_open() {

        if window.is_key_down(minifb::Key::Escape) {
            break;
        }

            // }
        match frame_rx.try_recv() {
            Ok(frame) => {
                window.update_with_buffer(frame.as_slice(), 160, 144) .unwrap(); 

                let fps = 1000 / last_frame.elapsed().as_millis();
                print!("Updates each {}FPS   \r", fps);
                last_frame = Instant::now();
            }
            _ => { } 
        };

        for (k, s) in &keys {
            if window.is_key_down(*k) {
                input_tx.send((*s, true)).unwrap();
            } else {
                input_tx.send((*s, false)).unwrap();
            }
        }

        window.update();
    }

    Ok(())
    }

