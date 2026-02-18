use crate::{apu::Apu, cart::Cartridge, ppu::Ppu};

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
    ppu: Ppu,
    apu: Apu,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: [0; 2048],
            cart: None,
            ppu: Ppu::new(),
            apu: Apu {},
        }
    }

    pub fn insert_cart(&mut self, cart: Cartridge) {
        self.ppu.load_patterns(cart.get_ppu_data());
        self.cart = Some(cart);
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize % 2048] = data, // RAM
            0x2000..=0x3FFF => self.ppu.write(addr),                  // PPU-REG
            0x4000..=0x401F => self.apu.write(addr),                  // APU
            0x4020..=0xFFFF => match &mut self.cart {
                Some(cart) => cart.write(addr as usize, data),
                None => panic!("No cart"),
            },
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize % 2048], // RAM
            0x2000..=0x3FFF => self.ppu.read(addr),            // PPU-REG
            0x4000..=0x401F => self.apu.read(addr),            // APU
            0x4020..=0xFFFF => match &self.cart {
                // CART ADDRESSES
                Some(cart) => cart.read_mem(addr as usize),
                None => panic!("No cart"),
            },
            _ => panic!("Unsupported address: {}", addr),
        }
    }
}
