use core::System;
use core::cartridge::Cartridge;
use core::Display;


struct DummyDisplay {}
impl Display for DummyDisplay { }

fn main() {
    let cartridge =
        // match Cartridge::from_path("roms/gb-test-roms/cpu_instrs/cpu_instrs.gb") {
        match Cartridge::from_path("roms/tetris.gb") {
            Ok(cartridge) => cartridge,
            _ => panic!("Error!"),
        };

    let display = Box::new(DummyDisplay{});

    let mut system = System::new(cartridge, display, true);

    loop {
        system.step();
    }
}
