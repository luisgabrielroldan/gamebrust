use crate::memory::Memory;

pub struct PPU {

}

impl PPU {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Memory for PPU {
    fn read(&self, _a: u16) -> u8 { 0xFF }
    fn write(&mut self, _addr: u16, _v: u8) { }
}
