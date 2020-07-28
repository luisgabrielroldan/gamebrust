pub mod mbc;

use self::mbc::RomOnly;
use self::mbc::MBC1;
use super::memory::Memory;
use std::fs::File;
use std::io::Read;

const KB: usize = 1024;
const MB: usize = 1024 * 1024;

pub struct Cartridge {
    mbc: Box<dyn Memory>,
    header: Header
}

impl Cartridge {
    pub fn from_path(rom_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(rom_path)?;
        let mut rom_data = Vec::new();
        file.read_to_end(&mut rom_data)?;

        let header = Header::read(&rom_data);

        let mbc: Box<dyn Memory> = match header.mbc_type {
            0x00 => Box::new(RomOnly::new(&header, rom_data)),
            0x01 => Box::new(MBC1::new(&header, rom_data)),
            t => panic!("Unsupported cartridge type: {:02x}", t),
        };

        Ok(Self {
            mbc: mbc,
            header: header,
        })
    }

    pub fn get_header(&self) -> Header {
        self.header.clone()
    }
}

impl Memory for Cartridge {
    fn read(&self, addr: u16) -> u8 { self.mbc.read(addr) }
    fn write(&mut self, addr: u16, v: u8) { self.mbc.write(addr, v); }
}


#[derive(Debug, Clone, Copy)]
enum CGB {
    DMGCompatible,
    CGBOnly,
    None
}

#[derive(Debug, Clone)]
pub struct Header {
    title: String,
    cgb: CGB,
    sgb: bool,
    mbc_type: u8,
    rom_size: usize,
    rom_banks: usize,
    ram_size: usize,
    ram_banks: usize,
}

impl Header {
    pub fn read(rom_data: &Vec<u8>) -> Self {
        let (ram_size, ram_banks) = Header::read_ram_size(rom_data);
        let (rom_size, rom_banks) = Header::read_ram_size(rom_data);
        let cgb = Header::read_cgb(rom_data);

        Self {
            title: Header::read_title(rom_data),
            cgb: cgb,
            sgb: Header::read_sgb(rom_data),
            mbc_type: rom_data[0x147],
            rom_size: rom_size,
            rom_banks: rom_banks,
            ram_size: ram_size,
            ram_banks: ram_banks,
        }
    }

    fn read_title(rom_data: &Vec<u8>) -> String {
        let title_size =
            match rom_data[0x143] & 0x80 {
                0x80 => 11,
                _ => 16,
            };

        let mut title = String::with_capacity(title_size as usize);

        for i in 0..title_size {
            match rom_data[0x134 + i] {
                0 => break,
                v => title.push(v as char),
            }
        }

        title

    }

    fn read_sgb(rom_data: &Vec<u8>) -> bool {
        rom_data[0x146] == 0x03
    }

    fn read_cgb(rom_data: &Vec<u8>) -> CGB {
        match rom_data[0x143] {
            0x80 => CGB::DMGCompatible,
            0xC0 => CGB::CGBOnly,
            _ => CGB::None,
        }
    }

    fn read_rom_size(rom_data: &Vec<u8>) -> (usize, usize) {
        match rom_data[0x0148] {
            0x00 => (32 * KB, 0),
            0x01 => (64 * KB, 4),
            0x02 => (128 * KB, 8),
            0x03 => (256 * KB, 16),
            0x04 => (512 * KB, 32),
            0x05 => (1 * MB, 64),
            0x06 => (2 * MB, 128),
            0x07 => (4 * MB, 256),
            0x08 => (8 * MB, 512),
            0x52 => (72 * 16 * KB, 72),
            0x53 => (80 * 16 * KB, 72),
            0x54 => (96 * 16 * KB, 72),
            n => panic!("Invalid rom size: 0x{:02x}", n),
        }
    }

    fn read_ram_size(rom_data: &Vec<u8>) -> (usize, usize) {
        match rom_data[0x0149] {
            0x00 => (0, 0),
            0x01 => (2 * KB, 1),
            0x02 => (8 * KB, 1),
            0x03 => (32 * KB, 4),
            0x04 => (128 * KB, 16),
            0x05 => (64 * KB, 8),
            n => panic!("Invalid ram size: 0x{:02x}", n),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_rom() {
        let cartridge =
            match Cartridge::from_path("../roms/pacman.gb") {
                Ok(cartridge) => cartridge,
                _ => panic!("Error!"),
            };

        let header = cartridge.get_header();

        assert_eq!(header.mbc_type, 1);
    }
}
