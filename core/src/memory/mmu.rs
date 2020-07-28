use crate::io::timer::Timer;
use crate::memory::Memory;
use crate::cartridge::Cartridge;
use crate::ppu::PPU;

pub struct MMU {
    intf: u8,
    cartridge: Cartridge,
    timer: Timer,
    ppu: PPU
}

#[allow(dead_code)]
impl MMU {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            intf: 0,
            cartridge: cartridge,
            timer: Timer::new(),
            ppu: PPU::new(),
        }
    }

    pub fn step(&mut self, ticks: u32) {
        self.intf |= self.timer.step(ticks);
    }
}

impl Memory for MMU {
    fn r8(&self, _addr: u16) -> u8 {
        0
    }

    fn w8(&mut self, _addr: u16, _v: u8) {}
}
