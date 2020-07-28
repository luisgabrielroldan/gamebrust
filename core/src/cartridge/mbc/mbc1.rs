use crate::memory::Memory;
use crate::cartridge::Header;

enum BankMode {
    Rom2MbRam8Kb,
    Rom512KbRam32Kb,
}

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    bank_mode: BankMode,
    bank: u8,
    ram_enabled: bool,
}

impl MBC1 {
    pub fn new(header: &Header, rom: Vec<u8>) -> Self {
        MBC1 {
            rom: rom,
            ram: vec![0; header.ram_size],
            bank_mode: BankMode::Rom2MbRam8Kb,
            bank: 1,
            ram_enabled: false,
        }
    }

    fn rom_bank(&self) -> usize {
        let n = match self.bank_mode {
            BankMode::Rom2MbRam8Kb => self.bank & 0x7F,
            BankMode::Rom512KbRam32Kb => self.bank & 0x1F,
        };
        n as usize
    }

    fn ram_bank(&self) -> usize {
        let n = match self.bank_mode {
            BankMode::Rom2MbRam8Kb => 0,
            BankMode::Rom512KbRam32Kb => (self.bank & 0x60) >> 5,
        };
        n as usize
    }
}

impl Memory for MBC1 {
    fn r8(&self, addr: u16) -> u8 {
        match addr {
            // 0000-3FFF - ROM Bank 00/20/40/60
            0x0000..=0x3FFF => self.rom[addr as usize],
            // 4000-7FFF - ROM Bank 01-7F
            0x4000..=0x7FFF => {
                let i = self.rom_bank() * 0x4000 + addr as usize - 0x4000;
                // if i > self.rom.len() { return 0xFF }
                self.rom[i]
            }
            // A000-BFFF - RAM Bank 00-03, if any
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let i = self.ram_bank() * 0x2000 + addr as usize - 0xa000;
                    self.ram[i]
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn w8(&mut self, addr: u16, v: u8) {
        match addr {
            // A000-BFFF - RAM Bank 00-03, if any
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let i = self.ram_bank() * 0x2000 + addr as usize - 0xa000;
                    self.ram[i] = v;
                }
            }
            // 0000-1FFF - RAM Enable
            0x0000..=0x1FFF => {
                self.ram_enabled = v & 0x0f == 0x0A;
            }
            // 2000-3FFF - ROM Bank Number
            0x2000..=0x3FFF => {
                let n = v & 0x1F;
                let n = match n {
                    0x00 => 0x01,
                    _ => n,
                };
                self.bank = (self.bank & 0x60) | n;
            }
            // 4000-5FFF - RAM Bank Number - or - Upper Bits of ROM Bank Number
            0x4000..=0x5FFF => {
                let n = v & 0x03;
                self.bank = self.bank & 0x9F | (n << 5)
            }
            // 6000-7FFF - Banking Mode Select
            0x6000..=0x7FFF => match v & 1 {
                0x00 => self.bank_mode = BankMode::Rom2MbRam8Kb,
                0x01 => self.bank_mode = BankMode::Rom512KbRam32Kb,
                n => panic!("MBC1: Invalid cartridge type {:02X}", n),
            },
            _ => {}
        }
    }
}
