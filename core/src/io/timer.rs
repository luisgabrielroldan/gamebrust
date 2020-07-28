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
    divider_ref: u32,
    div: u8,
    counter_ref: u32,
    counter: u8,
    modulo: u8,
    step: u32,
    enabled: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            divider_ref: 0,
            div: 0,
            counter_ref: 0,
            counter: 0,
            modulo: 0,
            enabled: true,
            step: 256,
        }
    }

    pub fn get_div(&self) -> u8 {
        self.div
    }
    pub fn get_counter(&self) -> u8 {
        self.counter
    }
    pub fn get_modulo(&self) -> u8 {
        self.modulo
    }
    pub fn get_enabled(&self) -> bool {
        self.enabled
    }
    pub fn get_divider(&self) -> Divider {
        match self.step {
            16 => Divider::By16,
            64 => Divider::By64,
            256 => Divider::By256,
            1024 => Divider::By1024,
            _ => panic!("TIMER: Unknown divider value!"),
        }
    }

    pub fn set_div(&mut self, _: u8) {
        self.div = 0;
    }
    pub fn set_counter(&mut self, v: u8) {
        self.counter = v;
    }
    pub fn set_modulo(&mut self, v: u8) {
        self.modulo = v;
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn set_divider(&mut self, divider: Divider) {
        self.step = divider as u32;
    }

    pub fn step(&mut self, ticks: u32) -> u8 {
        let mut result = 0;

        self.divider_ref += ticks;

        if self.divider_ref >= 256 {
            self.div = self.div.wrapping_add(1);
            self.divider_ref -= 256;
        }

        if !self.enabled {
            return 0;
        }

        self.counter_ref += ticks;

        while self.counter_ref >= self.step {
            self.counter = self.counter.wrapping_add(1);

            if self.counter == 0 {
                self.counter = self.modulo;

                result = io::intf_raise(0, io::Flag::Timer);
            }

            self.counter_ref -= self.step;
        }

        return result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupt_trigger() {
        let mut timer = Timer::new();
        let int = timer.step(256 * 255);
        assert_eq!(int, 0);
        let int = timer.step(256);
        assert_eq!(int, (1 << io::Flag::Timer as u8));
    }
}
