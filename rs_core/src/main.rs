pub mod bus;
pub mod cart;
pub mod cpu;
pub mod nes;


use crate::nes::Nes;

fn main() {
    let mut nes: Nes = Nes::new();

    let _ = nes.load_cart("nestest.nes");
    let mut i = 0;
    while i < 26554 {
        nes.step();
        i += 1;
    }

    nes.fetch_test_result();
}
