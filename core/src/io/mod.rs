pub mod timer;

pub enum Flag {
    VBlank = 0,
    LCDStat = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

pub fn intf_raise(flags: u8, flag: Flag) -> u8{
    flags | (1 << (flag as u8))
}
