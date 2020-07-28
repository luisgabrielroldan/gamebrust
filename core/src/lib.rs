mod cpu;
mod io;
pub mod cartridge;
mod memory;
mod ppu;

use cpu::CPU;
use memory::MMU;
use cartridge::Cartridge;

pub struct System {
    cpu: CPU,
    mmu: MMU,
}

#[allow(dead_code)]
impl System {
    pub fn new(cartridge: Cartridge, bootroom: bool) -> Self {
        let cpu = 
            match bootroom {
                true => CPU::new(),
                false => CPU::armed(),
            };

        Self {
            cpu: cpu,
            mmu: MMU::new(cartridge, bootroom),
        }
    }

    pub fn step(&mut self) {
        let ticks = self.cpu.step(&mut self.mmu);
        self.mmu.step(ticks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_system() {
        let cartridge =
            match Cartridge::from_path("../roms/tetris.gb") {
                Ok(cartridge) => cartridge,
                _ => panic!("Error!"),
            };

        System::new(cartridge, true);
    }
}
