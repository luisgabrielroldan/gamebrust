use crate::io::timer::Timer;
use crate::io::joypad::{Joypad, JoypadAdapter};
use crate::memory::Memory;
use crate::memory::Ram;
use crate::memory::bootrom::DMG1;
use crate::cartridge::Cartridge;
use crate::ppu::PPU;
use crate::Display;

struct OAMDma {
    active: bool,
    from: u16,
    index: u16,
}

impl OAMDma {
    pub fn new() -> Self {
        Self {
            active: false,
            from: 0,
            index: 0,
        }
    }

    pub fn start(&mut self, from: u8) {
        self.active = true;
        self.index = 0;
        self.from = (from as u16) << 8;
    }
}

pub struct MMU {
    intfs: u8,
    inte: u8,
    bootrom: bool,
    cartridge: Cartridge,
    timer: Timer,
    joypad: Joypad,
    ppu: PPU,
    wram: Ram,
    zram: Ram,
    sb: u8,
    oam_dma: OAMDma,
}

#[allow(dead_code)]
impl MMU {
    pub fn new(cartridge: Cartridge, display: Box<dyn Display>, bootrom: bool) -> Self {
        Self {
            intfs: 0,
            inte: 0,
            bootrom: bootrom,
            cartridge: cartridge,
            joypad: Joypad::new(),
            timer: Timer::new(),
            ppu: PPU::new(display),
            wram: Ram::new(0x8000),
            zram: Ram::new(0x7F),
            sb: 0,
            oam_dma: OAMDma::new(),
        }
    }

    pub fn step(&mut self, ticks: u32) {

        self.handle_oam_dma(ticks);

        self.intfs |= self.timer.step(ticks);
        self.intfs |= self.ppu.step(ticks);
        self.intfs |= self.joypad.step();

        self.intfs |= 0xE0;
    }

    pub fn get_joypad_adapter(&mut self) -> &mut dyn JoypadAdapter {
        &mut self.joypad
    }

      fn fast_oam_dma(&mut self, value: u8) {
        let base = (value as u16) << 8;
        for i in 0 .. 0xA0 {
            let b = self.read(base + i);
            self.write(0xFE00 + i, b);
        }
    }

    fn handle_oam_dma(&mut self, ticks: u32) {
        if !self.oam_dma.active { return; }

        let cycles = (ticks / 4) as u16;

        let count  = 
            if 0x8F - self.oam_dma.index > cycles {
                cycles
            } else {
                0x8F - self.oam_dma.index
            };

        for i in 0..count {
            let v = self.read(self.oam_dma.from + self.oam_dma.index + i);
            self.write(0xFE00 + self.oam_dma.index + i, v);
        }

        self.oam_dma.index += count;

        if self.oam_dma.index == 0x8F {
            self.oam_dma.active = false;
        }
    }

    fn io_read(&self, addr: u16) -> u8 {
        match addr {
            0xFF00 => self.joypad.read(),
            0xFF01..=0xFF02 => 0xFF,
            0xFF04 => self.timer.get_div(),
            0xFF05 => self.timer.get_tima(),
            0xFF06 => self.timer.get_tma(),
            0xFF07 => self.timer.get_tac(),
            0xFF0F => self.intfs,
            0xFF10..=0xFF3F => 0, // TODO: Implement sound someday...
            0xFF40..=0xFF4F => self.ppu.read(addr),
            0xFF50 => { if self.bootrom { 1 } else { 0 } }
            // 0xFF51..=0xFF55 => 0, // TODO DMA 
            0xFF68..=0xFF6B => self.ppu.read(addr),
            0xFFFF => self.inte,
            _ => { println!("Warning: Attempt to READ from unmapped IO area: 0x{:04X}", addr); 0xFF }
        }
    }

    fn io_write(&mut self, addr: u16, v: u8) {
        match addr {
            0xFF00 => self.joypad.write(v),
            0xFF01 => { self.sb = v; }
            0xFF02 => { if v == 0x81 { print!("{}", self.sb as char) } }
            0xFF04 => self.timer.set_div(v),
            0xFF05 => self.timer.set_tima(v),
            0xFF06 => self.timer.set_tma(v),
            0xFF07 => self.timer.set_tac(v),
            0xFF0F => { self.intfs = v; }
            0xFF10..=0xFF3F => {} // TODO: Implement sound someday...
            0xFF46 => { self.fast_oam_dma(v); } //self.oam_dma.start(v),
            0xFF40..=0xFF4F => self.ppu.write(addr, v),
            0xFF50 => { if (v & 1) == 1 { self.bootrom = false } }
            // 0xFF51..=0xFF55 => {} // DMA CGB
            0xFF68..=0xFF6B => self.ppu.write(addr, v),
            0xFFFF => self.inte = v,
            _ => { println!("Warning: Attempt to WRITE on unmapped IO area: 0x{:04X}", addr); }
        }
    }

}

impl Memory for MMU {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x000..=0x7FFF => {
                if self.bootrom && addr < 0x100 {
                    DMG1[addr as usize]
                } else {
                    self.cartridge.read(addr)
                }
            }
            0x8000..=0x9FFF => self.ppu.read(addr),
            0xA000..=0xBFFF => self.cartridge.read(addr),
            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram.read(addr & 0x0FFF),
            0xD000..=0xDFFF | 0xF000..=0xFDFF => self.wram.read(0x1000 | (addr & 0x0FFF)),
            0xFE00..=0xFE9F => self.ppu.read(addr),
            0xFEA0..=0xFEFF => 0xFF, // Not Used
            0xFF00..=0xFF7F => self.io_read(addr),
            0xFF80..=0xFFFE => self.zram.read(addr - 0xFF80),
            0xFFFF => self.io_read(addr),
        }
    }

    fn write(&mut self, addr: u16, v: u8) {
        match addr {
            0x000..=0x7FFF => self.cartridge.write(addr, v),
            0x8000..=0x9FFF => self.ppu.write(addr, v),
            0xA000..=0xBFFF => self.cartridge.write(addr, v),
            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram.write(addr & 0x0FFF, v),
            0xD000..=0xDFFF | 0xF000..=0xFDFF => self.wram.write(0x1000 | (addr & 0x0FFF), v),
            0xFE00..=0xFE9F => self.ppu.write(addr, v),
            0xFEA0..=0xFEFF => { /* Not Used */ }
            0xFF00..=0xFF7F => self.io_write(addr, v),
            0xFF80..=0xFFFE => self.zram.write(addr - 0xFF80, v),
            0xFFFF => self.io_write(addr, v),
        };

    }
}
