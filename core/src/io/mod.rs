pub enum Flag {
    VBlank = 0,
    LCDStat = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

pub struct IntFlag {
    pub value: u8,
}

impl IntFlag {
    pub fn from(value: u8) -> Self {
        Self { value }
    }

    pub fn raise(&mut self, flag: Flag) {
        self.value |= 1 << (flag as u8);
    }
}
