use core::System;
use core::cartridge::Cartridge;


fn main() {
    let cartridge =
        match Cartridge::from_path("roms/tetris.gb") {
            Ok(cartridge) => cartridge,
            _ => panic!("Error!"),
        };

    let mut system = System::new(cartridge);

    loop {
        system.step();
    }
}
