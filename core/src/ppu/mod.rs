pub mod sprite;
use crate::memory::{Memory, Ram};
use crate::io;
use crate::Display;

const SCREEN_W: u16 = 160;
const SCREEN_H: u16 = 144;

const VRAM_SIZE: usize = 0x4000;
const VOAM_SIZE: usize = 0xA0;

pub struct PPU {
    clock: u32,
    framebuffer: Vec<u32>,
    display: Box<dyn Display>,
    vram: Ram,
    voam: Ram,

    // The LY indicates the vertical line to which the present data is
    // transferred to the LCD Driver. The LY can take on any value between 0
    // through 153. The values between 144 and 153 indicate the V-Blank period.
    ly: u8,
    lyc_inte: bool,
    oam_inte: bool,
    vblank_inte: bool,
    hblank_inte: bool,
    lyc: u8,
    mode: Mode,
    scx: u8,
    scy: u8,
    wx: u8,
    wy: u8,
    bgp: Palette, // BGP - BG Palette Data (R/W) - Non CGB Mode Only
    obp0: Palette, // OBP0 - Object Palette 0 Data (R/W) - Non CGB Mode Only
    obp1: Palette, // OBP1 - Object Palette 1 Data (R/W) - Non CGB Mode Only

    lcd_on: bool,
    window_map: TileMap,
    window_on: bool,
    tile_data: TileSet,
    background_map: TileMap,
    sprite_size: SpriteSize,
    sprites_enabled: bool,

    // LCDC.0 has different meanings depending on Gameboy type and Mode:
    // Monochrome Gameboy, SGB and CGB in Non-CGB Mode: BG Display
    //  When Bit 0 is cleared, both background and window become blank (white),
    //  and the Window Display Bit is ignored in that case. Only Sprites may
    //  still be displayed (if enabled in Bit 1).
    //
    // CGB in CGB Mode: BG and Window Master Priority
    //  When Bit 0 is cleared, the background and window lose their priority
    //  the sprites will be always displayed on top of background and window,
    //  independently of the priority flags in OAM and BG Map attributes.
    lcdc0: bool,
}

impl PPU {
    pub fn new(display: Box<dyn Display>) -> Self {
        Self {
            clock: 0,
            display: display,
            framebuffer: vec![0xFFFFFF; (SCREEN_W * SCREEN_H) as usize],
            vram: Ram::new(VRAM_SIZE),
            voam: Ram::new(VOAM_SIZE),
            mode: Mode::HBlank,
            lcdc0: true,
            ly: 0,
            lyc: 0,
            lyc_inte: false,
            oam_inte: false,
            vblank_inte: false,
            hblank_inte: false,
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
            window_map: TileMap::Low,
            window_on: false,
            tile_data: TileSet::Set1,
            background_map: TileMap::High,
            bgp: Palette::from(0),
            obp0: Palette::from(0),
            obp1: Palette::from(0),
            lcd_on: true,
            sprite_size: SpriteSize::S8x8,
            sprites_enabled: false,
        }
    }

    pub fn step(&mut self, ticks: u32) -> u8 {
        let mut intfs: u8 = 0;

        self.clock += ticks;

        match self.mode {
            Mode::OAMSearch => {
                // Mode: 2
                if self.clock >= 80 {
                    self.clock -= 80;
                    intfs |= self.change_mode(Mode::Transfer);
                }
            }

            Mode::Transfer => {
                // Mode: 3
                if self.clock >= 172 {
                    self.clock -= 172;
                    intfs |= self.change_mode(Mode::HBlank);
                }
            }

            Mode::HBlank => {
                // Mode: 0
                if self.clock >= 204 {
                    self.clock -= 204;
                    self.ly = self.ly.wrapping_add(1);
                    intfs |= self.check_lyc();

                    if self.ly > 143 {
                        intfs |= self.change_mode(Mode::VBlank);
                    } else {
                        intfs |= self.change_mode(Mode::OAMSearch);
                    }
                }
            }

            Mode::VBlank => {
                // Mode: 1
                if self.clock >= 456 {
                    self.clock -= 456;
                    self.ly = self.ly.wrapping_add(1);
                    intfs |= self.check_lyc();

                    if self.ly > 153 {
                        self.ly = 0;
                        intfs |= self.change_mode(Mode::OAMSearch);
                    }
                }
            }
        }

        intfs
    }

    fn change_mode(&mut self, next: Mode) -> u8 {
        let mut intfs = 0;
        self.mode = next;

        if match self.mode {
            Mode::OAMSearch => self.oam_inte,
                Mode::HBlank => {
                    // self.render_line();
                    self.hblank_inte
                }
            Mode::VBlank => {
                self.display.update(&self.framebuffer);

                intfs = io::intf_raise(0, io::Flag::VBlank);
                self.vblank_inte
            }
            _ => false,
        } {
            intfs = io::intf_raise(0, io::Flag::LCDStat);
        }

        intfs
    }

    fn check_lyc(&mut self) -> u8 {
        if self.lyc_inte && self.ly == self.lyc {
            io::intf_raise(0, io::Flag::LCDStat)
        } else {
            0
        }
    }

    fn get_lcdc(&self) -> u8 {
        (if self.lcd_on { 1 << 7 } else { 0 }) |
            (if self.window_map == TileMap::High { 1 << 6 } else { 0 }) |
            (if self.window_on { 1 << 5 } else { 0 }) |
            (if self.tile_data == TileSet::Set1 { 1 << 4 } else { 0 }) |
            (if self.background_map == TileMap::High { 1 << 3 } else { 0 }) |
            (if self.sprite_size == SpriteSize::S8x16 { 1 << 2 } else { 0 }) |
            (if self.sprites_enabled { 1 << 1 } else { 0 }) |
            (if self.lcdc0 { 1 << 0} else { 0 })
    }

    fn get_stat(&self) -> u8 {
        (if self.lyc_inte { 1 << 6 } else { 0 }) |
            (if self.oam_inte { 1 << 5 } else { 0 }) |
            (if self.vblank_inte { 1 << 4 } else { 0 }) |
            (if self.hblank_inte { 1 << 3 } else { 0 }) |
            (if self.ly == self.lyc { 1 << 2 } else { 0 }) |
            (self.mode as u8)
    }

    fn set_lcdc(&mut self, v: u8) {
        self.lcd_on = (v & (1 << 7)) != 0;
        self.window_map = if (v & (1 << 6)) != 0 { TileMap::High } else { TileMap::Low };
        self.window_on = (v & (1 << 5)) != 0;
        self.tile_data = if (v & (1 << 4)) != 0 { TileSet::Set1 } else { TileSet::Set2 };
        self.background_map = if (v & (1 << 3)) != 0 { TileMap::High } else { TileMap::Low };
        self.sprite_size = if (v & (1 << 2)) != 0 { SpriteSize::S8x16 } else { SpriteSize::S8x8 };
        self.sprites_enabled = (v & (1 << 1)) != 0;
        self.lcdc0 = (v & 1) != 0;
    }

    fn set_stat(&mut self, v: u8) {
        self.lyc_inte = (v & (1 << 6)) != 0;
        self.oam_inte = (v & (1 << 5)) != 0;
        self.vblank_inte = (v & (1 << 4)) != 0;
        self.hblank_inte = (v & (1 << 3)) != 0;
    }
}

impl Memory for PPU {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.vram.read(addr - 0x8000),
            0xFE00 ..= 0xFE9F => self.voam.read(addr - 0xFE00),
            0xFF40 => self.get_lcdc(),
            0xFF41 => self.get_stat(),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp.into(),
            0xFF48 => self.obp0.into(),
            0xFF49 => self.obp1.into(),
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => { println!("Warning: Attempt to READ from unmapped PPU area: 0x{:04X}", addr); 0xFF }
        }
    }

    fn write(&mut self, addr: u16, v: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram.write(addr - 0x8000, v),
            0xFE00 ..= 0xFE9F => self.voam.write(addr - 0xFE00, v),
            0xFF40 => self.set_lcdc(v),
            0xFF41 => self.set_stat(v),
            0xFF42 => self.scy = v,
            0xFF43 => self.scx = v,
            0xFF44 => { }
            0xFF45 => self.lyc = v,
            0xFF47 => { self.bgp = Palette::from(v); }
            0xFF48 => { self.obp0 = Palette::from(v); }
            0xFF49 => { self.obp1 = Palette::from(v); }
            0xFF4A => self.wy = v,
            0xFF4B => self.wx = v,
            _ => { println!("Warning: Attempt to WRITE 0x{:02X} on unmapped PPU area: 0x{:04X}", v, addr) }
        };
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OAMSearch = 2,
    Transfer = 3,
}

#[derive(PartialEq, Clone, Copy)]
enum TileSet {
    Set1, // Map at 0x8000..0x8FFF
    Set2, // Map at 0x8800..0x97FF
}

impl TileSet {
    pub fn base_addr(set: &TileSet) -> u16 {
        match set {
            TileSet::Set1 => 0x8000,
            TileSet::Set2 => 0x8800,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum TileMap {
    Low,  // Map at 0x9800..0x9BFF
    High, // Map at 0x9C00..0x9FFF
}

impl TileMap {
    pub fn base_addr(set: &TileMap) -> u16 {
        match set {
            TileMap::Low => 0x9800,
            TileMap::High => 0x9C00,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum SpriteSize {
    S8x8 = 8,
    S8x16 = 16,
}

#[derive(PartialEq, Clone, Copy)]
struct Palette {
    value: u8,
    render: [u32; 4],
}

impl std::convert::From<u8> for Palette {
    fn from(value: u8) -> Self {
        let mut render: [u32; 4] = [0; 4];

        for i in 0..4 {
            let gray = match (value >> (i * 2)) & 0x03 {
                0 => 255,
                1 => 192,
                2 => 96,
                _ => 0,
            };

            render[i] = (gray << 24) | (gray << 16) | (gray << 8) | gray;
        }

        Self {
            value: value,
            render: render
        }
    }
}

impl std::convert::Into<u8> for Palette { fn into(self) -> u8 { self.value } }

impl Palette {
    pub fn to_rgb(&self, color_index: u8) -> u32 {
        self.render[color_index as usize]
    }
}
