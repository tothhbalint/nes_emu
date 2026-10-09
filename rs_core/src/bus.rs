use crate::cart::Cartridge;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BusError {
    #[error("Memory Access error:{0}")]
    InvalidRam(String),
    #[error("Memory Access error:{0}")]
    InvalidCart(String),
}

pub struct Bus {
    ram: [u8; 2048],
    cart: Option<Cartridge>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: [0; 2048],
            cart: None,
        }
    }

    pub fn insert_cart(&mut self, cart: Cartridge) {
        self.cart = Some(cart);
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize % 2048] = data, // RAM (mirrored)
            0x4020..=0xFFFF => match &mut self.cart {
                Some(cart) => cart.write(addr as usize, data),
                None => panic!("No cart"),
            },
            _ => {}
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize % 2048], // RAM (mirrored)
            0x2000..=0x401F => 0x0,                            // PPU / APU / I-O
            0x4020..=0xFFFF => match &self.cart {
                Some(cart) => cart.read_prg(addr as usize),
                None => panic!("No cart"),
            },
        }
    }
}
