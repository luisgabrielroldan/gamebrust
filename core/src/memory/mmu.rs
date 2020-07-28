use crate::io::timer::Timer;
use crate::memory::Memory;
use crate::memory::Ram;
use crate::memory::bootrom::DMG1;
use crate::cartridge::Cartridge;
use crate::ppu::PPU;

pub struct MMU {
    intf: u8,
    inte: u8,
    bootrom: bool,
    cartridge: Cartridge,
    timer: Timer,
    ppu: PPU,
    wram: Ram,
    zram: Ram,
    sb: u8,
}

#[allow(dead_code)]
impl MMU {
    pub fn new(cartridge: Cartridge, bootrom: bool) -> Self {
        Self {
            intf: 0,
            inte: 0,
            bootrom: bootrom,
            cartridge: cartridge,
            timer: Timer::new(),
            ppu: PPU::new(),
            wram: Ram::new(0x8000),
            zram: Ram::new(0x7F),
            sb: 0,
        }
    }

    pub fn step(&mut self, ticks: u32) {
        self.intf |= self.timer.step(ticks);
        self.intf |= self.ppu.step(ticks);
    }

    fn io_read(&self, addr: u16) -> u8 {
        match addr {
            // 0xFF00 => self.joypad.read(),
            0xFF01..=0xFF02 => 0xFF,
            0xFF04 => self.timer.get_div(),
            0xFF05 => self.timer.get_counter(),
            0xFF06 => self.timer.get_modulo(),
            0xFF07 => self.timer.get_tac(),
            0xFF0F => self.intf,
            0xFF10..=0xFF3F => 0, // TODO: Implement sound someday...
            0xFF40..=0xFF4F => self.ppu.read(addr),
            0xFF50 => { if self.bootrom { 1 } else { 0 } }
            // 0xFF51..=0xFF55 => 0, // TODO DMA 
            0xFF68..=0xFF6B => self.ppu.read(addr),
            0xFFFF => self.inte,
            _ => { println!("Warning: MMU: Attempt to READ from unmapped IO area: 0x{:04X}", addr); 0xFF }
        }
    }

    fn io_write(&mut self, addr: u16, v: u8) {
        match addr {
            // 0xFF00 => self.joypad.write(v),
            0xFF01 => { self.sb = v; }
            0xFF02 => { if v == 0x81 { print!("{}", self.sb as char) } }
            0xFF04 => self.timer.set_div(v),
            0xFF05 => self.timer.set_counter(v),
            0xFF06 => self.timer.set_modulo(v),
            0xFF07 => self.timer.set_tac(v),
            0xFF0F => { self.intf = v; }
            0xFF10..=0xFF3F => {} // TODO: Implement sound someday...
            // 0xFF46 => self.oam_dma(v),
            0xFF40..=0xFF4F => self.ppu.write(addr, v),
            0xFF50 => { if (v & 1) == 1 { self.bootrom = false } }
            // 0xFF51..=0xFF55 => {} // DMA CGB
            0xFF68..=0xFF6B => self.ppu.write(addr, v),
            0xFFFF => self.inte = v,
            _ => { println!("Warning: MMU: Attempt to WRITE on unmapped IO area: 0x{:04X}", addr); }
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
            0xFF00..=0xFF7F => self.io_read(addr),
            0xFF80..=0xFFFE => self.zram.read(addr - 0xFF80),
            0xFFFF => self.io_read(addr),
            _ => { println!("Warning: MMU: Attempt to READ from unmapped area: 0x{:04X}", addr); 0xFF }
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
            0xFF00..=0xFF7F => self.io_write(addr, v),
            0xFF80..=0xFFFE => self.zram.write(addr - 0xFF80, v),
            0xFFFF => self.io_write(addr, v),
            _ => { println!("Warning: MMU: Attempt to WRITE on unmapped area: 0x{:04X}", addr); }
        };

    }
}
