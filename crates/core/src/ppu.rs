mod bus;
mod oam;
mod registers;

use bus::*;
use oam::Oam;
use registers::*;

use crate::cart::Mirroring;

const X_SIZE: usize = 283;
const Y_SIZE: usize = 242;
type RenderedImage = [[u8; X_SIZE]; Y_SIZE];

pub struct Ppu {
    pub regs: Registers,
    pub oam: Oam,
    bus: PpuBus,
    vram: u16,
    scroll_x: u8,
    scroll_y: u8,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            regs: Registers {
                ctrl: PPUCTRL::empty(),
                mask: PPUMASK::empty(),
                status: PPUSTATUS::empty(),
                v: 0,
                t: 0,
                x: 0,
                w: false,
            },
            bus: PpuBus::new(),
            oam: Oam::new(),
            vram: 0,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    pub fn load_chr(&mut self, chr: &[u8], mirroring: Mirroring) {
        self.bus.load(chr, mirroring);
    }

    #[inline]
    pub fn read_chr(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    #[inline]
    pub fn reg_read(&self, idx: u8) -> u8 {
        let r = &self.regs;
        match idx & 0x07 {
            0 => r.ctrl.bits(),
            1 => r.mask.bits(),
            2 => r.status.bits(),
            3 => self.oam.addr(),
            4 => self.oam.peek(),
            5 => r.x,
            6 => (r.v >> 8) as u8,
            _ => self.vram as u8,
        }
    }
    #[inline]
    pub fn reg_write(&mut self, idx: u8, val: u8) {
        let r = &mut self.regs;
        match idx & 0x07 {
            0 => r.write_ctrl(val),
            1 => r.write_mask(val),
            3 => self.oam.set_addr(val),
            4 => self.oam.write(val),
            _ => self.vram = val as u16,
        }
    }

    pub fn render(&self) -> RenderedImage {
        [[0; X_SIZE]; Y_SIZE]
    }
}
