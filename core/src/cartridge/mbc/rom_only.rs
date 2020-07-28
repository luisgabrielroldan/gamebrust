use crate::memory::Memory;
use crate::cartridge::Header;

pub struct RomOnly {
    rom: Vec<u8>,
}

impl RomOnly {
    pub fn new(_header: &Header, rom: Vec<u8>) -> Self {
        RomOnly { rom: rom }
    }
}

impl Memory for RomOnly {
    fn r8(&self, a: u16) -> u8 {
        self.rom[a as usize]
    }

    fn w8(&mut self, _addr: u16, _v: u8) { }
}
