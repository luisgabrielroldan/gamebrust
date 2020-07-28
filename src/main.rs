use core::System;
use core::cartridge::Cartridge;


fn main() {
    let cartridge =
        match Cartridge::from_path("roms/gb-test-roms/cpu_instrs/cpu_instrs.gb") {
            Ok(cartridge) => cartridge,
            _ => panic!("Error!"),
        };

    let mut system = System::new(cartridge, true);

    loop {
        system.step();
    }
}
