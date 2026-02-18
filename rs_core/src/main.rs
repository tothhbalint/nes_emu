pub mod apu;
pub mod bus;
pub mod cart;
pub mod cpu;
pub mod nes;
pub mod ppu;

use crate::nes::Nes;

fn main() {
    let mut nes: Nes = Nes::new();

    nes.run_test();
}
