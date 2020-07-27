use crate::io::IntFlag;
use crate::memory::Memory;

pub struct MMU {
    intf: IntFlag,
}

#[allow(dead_code)]
impl MMU {
    pub fn new() -> Self {
        Self {
            intf: IntFlag::from(0),
        }
    }

    pub fn step(&mut self, _ticks: u32) {}
}

impl Memory for MMU {
    fn r8(&self, _addr: u16) -> u8 {
        0
    }

    fn w8(&mut self, _addr: u16, _v: u8) {}
}
