mod mmu;
mod bootrom;

pub use mmu::MMU;

pub trait Memory {
    fn read(&self, _addr: u16) -> u8;
    fn write(&mut self, _addr: u16, _v: u8);

    #[inline]
    fn read_word(&self, addr: u16) -> u16 {
        let l = self.read(addr);
        let h = self.read(addr.wrapping_add(1));

        (l as u16) | ((h as u16) << 8)
    }

    #[inline]
    fn write_word(&mut self, addr: u16, v: u16) {
        let h = (v >> 8) as u8;
        let l = (v & 0xFF) as u8;

        self.write(addr, l);
        self.write(addr.wrapping_add(1), h);
    }
}

pub struct Ram {
    data: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }
}

impl Memory for Ram {
    fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn write(&mut self, addr: u16, v: u8) {
        self.data[addr as usize] = v
    }
}
