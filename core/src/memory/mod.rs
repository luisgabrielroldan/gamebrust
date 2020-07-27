mod mmu;

pub use mmu::MMU;

pub trait Memory {
    fn r8(&self, _addr: u16) -> u8;
    fn w8(&mut self, _addr: u16, _v: u8);

    #[inline]
    fn r16(&self, addr: u16) -> u16 {
        let l = self.r8(addr);
        let h = self.r8(addr.wrapping_add(1));

        (l as u16) | ((h as u16) << 8)
    }

    #[inline]
    fn w16(&mut self, addr: u16, v: u16) {
        let h = (v >> 8) as u8;
        let l = (v & 0xFF) as u8;

        self.w8(addr, l);
        self.w8(addr.wrapping_add(1), h);
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
    fn r8(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn w8(&mut self, addr: u16, v: u8) {
        self.data[addr as usize] = v
    }
}
