mod bus;
mod oam;
mod palette;
mod registers;

#[cfg(test)]
mod tests;

use bus::*;
use oam::Oam;
use registers::*;

use crate::cart::Mirroring;

const X_SIZE: usize = 256;
const Y_SIZE: usize = 240;

pub type FrameBuffer = [[u8; X_SIZE]; Y_SIZE];

#[derive(Default)]
struct Nmi {
    nmi_line: bool,
    nmi_latch: bool,
}

impl Nmi {
    pub fn sample_line(&mut self, line: bool) {
        if line && !self.nmi_line {
            self.nmi_latch = true;
        }
        self.nmi_line = line;
    }

    pub fn take(&mut self) -> bool {
        std::mem::replace(&mut self.nmi_latch, false)
    }
}

pub struct Ppu {
    regs: Registers,
    oam: Oam,
    bus: PpuBus,
    nmi: Nmi,

    dots: u16,      // (0-341),
    scanlines: u16, // (0-261)
    open_bus: u8,
    buffer: FrameBuffer,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            regs: Registers::default(),
            bus: PpuBus::default(),
            oam: Oam::default(),
            nmi: Nmi::default(),

            dots: 0,
            scanlines: 261,
            open_bus: 0,
            buffer: [[0; X_SIZE]; Y_SIZE],
        }
    }

    pub fn take_nmi(&mut self) -> bool {
        self.nmi.take()
    }

    pub fn tick(&mut self) {
        self.dots += 1;

        let rendering_enabled = self
            .regs
            .mask
            .intersects(PPUMASK::BACKGROUND_RENDERING | PPUMASK::SPRITE_RENDERING);

        match (self.scanlines, self.dots) {
            (0..=239, 1) => render::scanline(
                &mut self.regs,
                &self.bus,
                &self.oam,
                &mut self.buffer,
                self.scanlines,
            ),

            (241, 1) => {
                self.regs.set_vblank();
                self.nmi.sample_line(self.regs.nmi());
            }

            (261, 1) => {
                self.regs.unset_vblank();
                self.nmi.sample_line(self.regs.nmi());
            }

            (0..=239, 256) | (261, 256) if rendering_enabled => self.regs.y_increment(),
            (0..=239, 257) | (261, 257) if rendering_enabled => self.regs.copy_horizontal(),
            (261, 280..=304) if rendering_enabled => self.regs.copy_vertical(),
            _ => {}
        }

        if self.dots == 341 {
            self.dots = 0;
            self.scanlines += 1;

            if self.scanlines == 262 {
                self.scanlines = 0;
            }
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
    pub fn reg_read(&mut self, idx: u8) -> u8 {
        let r = &mut self.regs;
        match idx & 0x07 {
            2 => {
                let value = r.read_status();
                self.nmi.sample_line(self.regs.nmi());
                value
            }
            4 => self.oam.peek(),
            7 => r.read_ppudata(&self.bus),
            _ => self.open_bus,
        }
    }
    #[inline]
    pub fn reg_write(&mut self, idx: u8, val: u8) {
        let r = &mut self.regs;
        self.open_bus = val;
        match idx & 0x07 {
            0 => {
                r.write_ctrl(val);
                self.nmi.sample_line(self.regs.nmi());
            }
            1 => r.write_mask(val),
            3 => self.oam.set_addr(val),
            4 => self.oam.write(val),
            5 => r.write_scroll(val),
            6 => r.write_addr(val),
            7 => r.write_ppudata(&mut self.bus, val),
            _ => {}
        }
    }

    pub fn render(&self) -> &FrameBuffer {
        &self.buffer
    }
}

mod render {
    use crate::ppu::{
        bus::PpuBus,
        oam::Oam,
        registers::{Registers, PPUCTRL, PPUSTATUS},
        FrameBuffer,
    };

    mod background;
    mod sprites;

    pub fn scanline(
        regs: &mut Registers,
        bus: &PpuBus,
        oam: &Oam,
        framebuffer: &mut FrameBuffer,
        scanline: u16,
    ) {
        if scanline > 240 {
            return;
        }

        let height = if regs.ctrl.contains(PPUCTRL::SPRITE_SIZE) {
            16
        } else {
            8
        };

        let (mask, overflow) = oam.select(scanline, height);
        let sprites = oam.sprites();

        background::render(regs, scanline, bus, framebuffer);

        // Sprite 0 hit check
        regs.status.set(
            PPUSTATUS::SPRITE_0,
            sprites::render(sprites, mask, regs, bus, framebuffer, scanline, height),
        );
    }
}
