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
        self.ram[addr as usize % 2048] = data;
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..0x2000 => self.ram[addr as usize % 2048], // RAM
            0x2000..0x4000 => 0x0,                            // PPU
            0x4000..0x4018 => 0x0,                            // APU/I/O
            0x4018..0x4020 => 0x0,                            // CPU TEST MODE
            0x4020..0x8000 => 0x0,                            //UNMAPPED
            0x8000..=0xFFFF => match &self.cart {
                Some(cart) => cart.read_prg(addr as usize),
                None => panic!("No cart"),
            },
            _ => panic!("Unsupported address: {}", addr),
        }
    }
}
