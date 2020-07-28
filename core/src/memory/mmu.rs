use crate::io::timer::Timer;
use crate::memory::Memory;

pub struct MMU {
    intf: u8,
    timer: Timer,
}

#[allow(dead_code)]
impl MMU {
    pub fn new() -> Self {
        Self {
            intf: 0,
            timer: Timer::new(),
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
