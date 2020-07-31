use crate::io;

#[derive(Debug)]
pub enum Divider {
    By16 = 16,
    By64 = 64,
    By256 = 256,
    By1024 = 1024,
}

#[derive(Debug)]
pub struct Timer {
    div_clock: u32,
    div: u8,
    tma_clock: u32,
    tima: u8,
    tma: u8,
    period: u32,
    enabled: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div_clock: 0,
            div: 0,
            tma_clock: 0,
            tima: 0,
            tma: 0,
            enabled: true,
            period: 256,
        }
    }

    pub fn get_div(&self) -> u8 {
        self.div
    }
    pub fn get_tima(&self) -> u8 {
        self.tima
    }
    pub fn get_tma(&self) -> u8 {
        self.tma
    }
    pub fn get_enabled(&self) -> bool {
        self.enabled
    }
    pub fn get_divider(&self) -> Divider {
        match self.period {
            16 => Divider::By16,
            64 => Divider::By64,
            256 => Divider::By256,
            1024 => Divider::By1024,
            _ => panic!("TIMER: Unknown divider value!"),
        }
    }

    pub fn get_tac(&self) -> u8 {
        (if self.get_enabled() { 1 << 2 } else { 0 })
            | (match self.get_divider() {
                Divider::By1024 => 0,
                Divider::By16 => 1,
                Divider::By64 => 2,
                Divider::By256 => 3,
            } as u8)
            | 0xF8
    }

    pub fn set_tac(&mut self, v: u8) {
        self.set_enabled((v & 0x04) != 0);

        let divider =
            match v & 0x03 {
                0 => Divider::By1024,
                1 => Divider::By16,
                2 => Divider::By64,
                _ => Divider::By256,
            } as u32;

        if divider != self.period {
            // println!("Set TAC: period={:?}, enabled={:?}", divider, self.enabled);
            self.period = divider;
            self.tma_clock = 0;
            self.tima = self.tma;
        }
    }

    pub fn set_div(&mut self, _: u8) {
        self.div = 0;
    }
    pub fn set_tima(&mut self, v: u8) {
        // println!("SET TIMA={}", v);
        self.tima = v;
    }
    pub fn set_tma(&mut self, v: u8) {
        self.tma = v;
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    fn set_divider(&mut self, divider: Divider) {
        self.period = divider as u32;
    }

    pub fn step(&mut self, ticks: u32) -> u8 {
        let mut result = 0;

        // Divider

        self.div_clock += ticks;

        if self.div_clock >= 256 {
            self.div = self.div.wrapping_add(1);
            self.div_clock -= 256;
        }

        if !self.enabled {
            return 0;
        }

        // Tima
        
        self.tma_clock += ticks / 4;

        while self.tma_clock >= self.period {
            self.tima = self.tima.wrapping_add(1);

            if self.tima == 0 {
                self.tima = self.tma;

                result = io::intf_raise(result, io::Flag::Timer);
            }

            self.tma_clock -= self.period;
        }

        return result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider() {
        let mut timer = Timer::new();
        for i in 0..255 { timer.step(1); }
        assert_eq!(timer.div, 0);
        timer.step(1);
        assert_eq!(timer.div, 1);
        for i in 0..1024 { timer.step(1); }
        assert_eq!(timer.div, 5);
    }

    #[test]
    fn interrupt_trigger() {
        let mut timer = Timer::new();
        let mut int = 0;
        timer.set_tac(0x05);
        timer.set_tima(0);
        for i in 0..1024 { int |= timer.step(4); }
        // println!("TIMA={}", timer.tima);
        // println!("INTF={}", int);
        // assert_eq!(int, 0);
        // for i in 0..500 { int = timer.step(4); }
        // let int = timer.step(500 * 4);
        // assert_eq!(int, 4);
        // let int = timer.step(256);
        // assert_eq!(int, (1 << io::Flag::Timer as u8));
    }
}
