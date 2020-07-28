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
    fn r8(&self, _a: u16) -> u8 { 0xFF }
    fn w8(&mut self, _addr: u16, _v: u8) { }
}
