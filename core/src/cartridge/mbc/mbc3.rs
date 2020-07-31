use crate::memory::Memory;
use crate::cartridge::Header;

pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
}

impl MBC3 {
    pub fn new(header: &Header, rom: Vec<u8>) -> Self {
        Self {
            rom: rom,
            ram: vec![0; header.ram_size],
            rom_bank: 1,
            ram_bank: 1,
            ram_enabled: false,
        }
    }
}

impl Memory for MBC3 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // 0000-3FFF - ROM Bank 00/20/40/60
            0x0000..=0x3FFF => self.rom[addr as usize],
            // 4000-7FFF - ROM Bank 01-7F
            0x4000..=0x7FFF => {
                let i = self.rom_bank * 0x4000 + addr as usize - 0x4000;
                // if i > self.rom.len() { return 0xFF }
                self.rom[i]
            }
            // A000-BFFF - RAM Bank 00-03, if any
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if self.ram_bank <= 0x03 {
                        let i = self.ram_bank * 0x2000 + addr as usize - 0xa000;
                        self.ram[i]
                    } else {
                        0
                        // Writed RTC
                    }
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, v: u8) {
        match addr {
            // A000-BFFF - RAM Bank 
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if self.ram_bank <= 0x03 {
                        let i = self.ram_bank * 0x2000 + addr as usize - 0xa000;
                        self.ram[i] = v;
                    } else {
                        // Write RTC
                    }
                }
            }
            // 0000-1FFF - RAM Enable
            0x0000..=0x1FFF => {
                self.ram_enabled = v & 0x0f == 0x0A;
            }
            // 2000-3FFF - ROM Bank Number
            0x2000..=0x3FFF => {
                let n = v & 0x7F;
                let n = match n {
                    0x00 => 0x01,
                    _ => n,
                };
                self.rom_bank = n as usize;
            }
            // 4000-5FFF - RAM Bank Number
            0x4000..=0x5FFF => {
                let n = v & 0x0F;
                self.ram_bank = n as usize;
            }
            // 6000-7FFF - Banking Mode Select
            0x6000..=0x7FFF =>  {
                // RTC
            },
            _ => {}
        }
    }
}
