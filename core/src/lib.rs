#[allow(dead_code)]
mod cpu;
#[allow(dead_code)]
mod io;
#[allow(dead_code)]
mod memory;

use cpu::CPU;
use memory::MMU;

struct System {
    cpu: CPU,
    mmu: MMU,
}

#[allow(dead_code)]
impl System {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            mmu: MMU::new(),
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
    fn it_works() {
        let system = System::new();
    }
}
