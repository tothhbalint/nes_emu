use crate::{cart::Cartridge, ppu::Ppu};

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
}

impl Bus {
    pub fn new(ppu: Ppu) -> Self {
        Bus {
            ram: [0; 2048],
            cart: None,
            ppu: ppu,
        }
    }

    pub fn insert_cart(&mut self, cart: Cartridge) {
        self.ppu.load_chr(cart.chr(), cart.mirroring());
        self.cart = Some(cart);
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            // 2 KB internal RAM, mirrored 4x
            0x0000..=0x1FFF => self.ram[addr as usize % 2048] = data,
            // PPU / APU / I-O registers: not modeled yet
            0x2000..=0x3FFF => {
                self.ppu.reg_write((addr & 0x0007) as u8, data);
            }
            0x4000..0x4014 => {}
            0x4014 => {
                self.oamdma(data);
            }
            0x4014..=0x4017 => {} // APU I/O registers
            0x4018..=0x401F => {} // CPU Test mode APU I/O
            // Expansion ($4020-$5FFF) + cartridge space ($6000-$FFFF)
            0x4020..=0xFFFF => match &mut self.cart {
                Some(cart) => cart.write(addr as usize, data),
                None => panic!("No cart"),
            },
        }
    }

    fn oamdma(&mut self, page: u8) {
        let mut addr = (page as u16) << 8;

        for _ in 0..256 {
            let data = self.read(addr);
            self.ppu.reg_write(4, data); // OAMDATA reg

            addr = addr.wrapping_add(1);
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize % 2048], // RAM (mirrored)
            0x2000..=0x3FFF => self.ppu.reg_read((addr & 0x0007) as u8),
            0x4000..=0x401F => 0x0, // APU/I-O
            0x4020..=0xFFFF => match &self.cart {
                Some(cart) => cart.read(addr as usize),
                None => panic!("No cart"),
            },
        }
    }
}
