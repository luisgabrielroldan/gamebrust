#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum R8 {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum R16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Debug, Clone, Copy)]
pub struct Flags {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,
    low: u8,
}

impl Flags {
    pub fn new() -> Self {
        Self {
            z: false,
            n: false,
            h: false,
            c: false,
            low: 0,
        }
    }

    pub fn set(&mut self, v: u8) {
        self.z = (v & (1 << 7)) != 0;
        self.n = (v & (1 << 6)) != 0;
        self.h = (v & (1 << 5)) != 0;
        self.c = (v & (1 << 4)) != 0;
        self.low = v & 0x0F;
    }

    pub fn to_u8(&self) -> u8 {
        (if self.z { 1 << 7 } else { 0 })
            | (if self.n { 1 << 6 } else { 0 })
            | (if self.h { 1 << 5 } else { 0 })
            | (if self.c { 1 << 4 } else { 0 })
            | (self.low & 0xF)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub flags: Flags,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            flags: Flags::new(),
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    pub fn get_r8(&mut self, reg: R8) -> u8 {
        match reg {
            R8::A => self.a,
            R8::B => self.b,
            R8::C => self.c,
            R8::D => self.d,
            R8::E => self.e,
            R8::H => self.h,
            R8::L => self.l,
            R8::F => self.flags.to_u8(),
        }
    }
    pub fn set_r8(&mut self, reg: R8, v: u8) {
        match reg {
            R8::A => self.a = v,
            R8::B => self.b = v,
            R8::C => self.c = v,
            R8::D => self.d = v,
            R8::E => self.e = v,
            R8::H => self.h = v,
            R8::L => self.l = v,
            R8::F => self.flags.set(v),
        };
    }

    pub fn set_r16(&mut self, reg: R16, v: u16) {
        let h = (v >> 8) as u8;
        let l = (v & 0xFF) as u8;

        match reg {
            R16::AF => {
                self.a = h;
                self.flags.set(l);
            }
            R16::BC => {
                self.b = h;
                self.c = l;
            }
            R16::DE => {
                self.d = h;
                self.e = l;
            }
            R16::HL => {
                self.h = h;
                self.l = l;
            }
            R16::SP => {
                self.sp = v;
            }
            R16::PC => {
                self.pc = v;
            }
        };
    }

    pub fn get_r16(&mut self, reg: R16) -> u16 {
        match reg {
            R16::SP => (self.sp),
            R16::PC => (self.pc),
            reg => {
                let (h, l) = match reg {
                    R16::AF => (self.a, self.flags.to_u8()),
                    R16::BC => (self.b, self.c),
                    R16::DE => (self.d, self.e),
                    R16::HL => (self.h, self.l),
                    _ => panic!("Invalid match! This should never happen."),
                };

                (h as u16) << 8 | (l as u16)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_check() {
        let mut regs = Registers::new();

        regs.set_r16(R16::AF, 0xBEEF);
        regs.set_r16(R16::BC, 0xBEEF);
        regs.set_r16(R16::DE, 0xBEEF);
        regs.set_r16(R16::HL, 0xBEEF);
        regs.set_r16(R16::SP, 0xBEEF);
        regs.set_r16(R16::PC, 0xBEEF);

        assert_eq!(0xBE, regs.a);
        assert_eq!(0xEF, regs.flags.to_u8());
        assert_eq!(0xBE, regs.b);
        assert_eq!(0xEF, regs.c);
        assert_eq!(0xBE, regs.d);
        assert_eq!(0xEF, regs.e);
        assert_eq!(0xBE, regs.h);
        assert_eq!(0xEF, regs.l);
        assert_eq!(0xBEEF, regs.sp);
        assert_eq!(0xBEEF, regs.pc);

        assert_eq!(0xBEEF, regs.get_r16(R16::AF));
        assert_eq!(0xBEEF, regs.get_r16(R16::BC));
        assert_eq!(0xBEEF, regs.get_r16(R16::DE));
        assert_eq!(0xBEEF, regs.get_r16(R16::HL));
        assert_eq!(0xBEEF, regs.get_r16(R16::SP));
        assert_eq!(0xBEEF, regs.get_r16(R16::PC));
    }
}
