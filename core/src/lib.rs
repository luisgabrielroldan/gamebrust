mod cpu;
pub mod io;
pub mod cartridge;
mod memory;
mod ppu;

use cpu::CPU;
use memory::MMU;
use cartridge::Cartridge;
use crate::io::joypad::JoypadAdapter;
use std::path::Path;

// pub const CLOCK_FREQUENCY: u32 = 4_194_304;
// pub const BATCH_TIME: u32 = 1;
// pub const BATCH_TICKS: u32 = (BATCH_TIME as f64 / (1000_f64 / CLOCK_FREQUENCY as f64)) as u32;

pub trait Display {
    fn update(&mut self, _framebuffer: &Vec<u32>) { }
}

pub struct System {
    cpu: CPU,
    mmu: MMU,
}

#[allow(dead_code)]
impl System {
    pub fn new(cartridge: Cartridge, display: Box<dyn Display>, bootroom: bool) -> Self {
        let cpu = 
            match bootroom {
                true => CPU::new(),
                false => CPU::armed(),
            };

        Self {
            cpu: cpu,
            mmu: MMU::new(cartridge, display, bootroom),
        }
    }

    pub fn step(&mut self) -> u32 {
        let ticks = self.cpu.step(&mut self.mmu);
        self.mmu.step(ticks);
        ticks
    }

    pub fn get_joypad_adapter(&mut self) -> &mut dyn JoypadAdapter {
        self.mmu.get_joypad_adapter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyDisplay {}

    impl Display for DummyDisplay {
    }

    #[test]
    fn create_system() {
        let cartridge =
            match Cartridge::from_path(Path::new("../roms/tetris.gb")) {
                Ok(cartridge) => cartridge,
                _ => panic!("Error!"),
            };

        let mut system = System::new(cartridge, Box::new(DummyDisplay{}), true);

        system.step();

        { 
            let joypad = system.get_joypad_adapter();
            joypad.pressed(crate::io::joypad::JoypadKey::Start);
        }

        system.step();
    }
}
