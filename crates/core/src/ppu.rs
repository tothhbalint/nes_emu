mod bus;
mod oam;
mod registers;

#[cfg(test)]
mod tests;

use bus::*;
use oam::{Oam, Sprite, SpriteByte};
use registers::*;

use crate::{cart::Mirroring, ppu::oam::SpriteByte::Attr};

const X_SIZE: usize = 256;
const Y_SIZE: usize = 240;

pub type FrameBuffer = [[u8; X_SIZE]; Y_SIZE];

pub struct Ppu {
    pub regs: Registers,
    pub oam: Oam,
    bus: PpuBus,
    nmi_line: bool,
    nmi_latch: bool,
    scroll_x: u8,
    scroll_y: u8,

    frame_count: u64,
    dots: u16,      // (0-341),
    scanlines: u16, // (0-261)
    open_bus: u8,
    buffer: FrameBuffer,
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
            nmi_line: false,
            nmi_latch: false,
            scroll_x: 0,
            scroll_y: 0,
            frame_count: 0,

            dots: 0,
            scanlines: 0,
            open_bus: 0,
            buffer: [[0; X_SIZE]; Y_SIZE],
        }
    }

    fn sample_nmi_line(&mut self) {
        let line = self.regs.status.contains(PPUSTATUS::VBLANK)
            && self.regs.ctrl.contains(PPUCTRL::NMI_ENABLE);
        if line && !self.nmi_line {
            self.nmi_latch = true;
        }
        self.nmi_line = line;
    }

    pub fn take_nmi(&mut self) -> bool {
        std::mem::take(&mut self.nmi_latch)
    }

    pub fn tick(&mut self) {
        self.dots += 1;
        if self.dots == 341 {
            self.dots = 0;
            self.render_scanline();
            self.scanlines += 1;
            if self.scanlines == 262 {
                self.scanlines = 0;
                self.advance_frame();
            }
        }

        match (self.scanlines, self.dots) {
            (241, 1) => {
                self.regs.status.insert(PPUSTATUS::VBLANK);
                self.sample_nmi_line();
            }
            (261, 1) => {
                self.regs.status.remove(PPUSTATUS::VBLANK);
                self.sample_nmi_line();
            }
            _ => {}
        }
    }

    pub fn load_chr(&mut self, chr: &[u8], mirroring: Mirroring) {
        self.bus.load(chr, mirroring);
    }

    #[inline]
    pub fn check_nmi(&self) -> bool {
        self.nmi_latch
    }

    #[inline]
    pub fn read_chr(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    #[inline]
    pub fn reg_read(&mut self, idx: u8) -> u8 {
        let r = &mut self.regs;
        match idx & 0x07 {
            2 => r.read_status(),
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
                self.sample_nmi_line();
            }
            1 => r.write_mask(val),
            3 => self.oam.set_addr(val),
            4 => self.oam.write(val),
            5 => r.write_ppuscroll(val),
            6 => r.write_ppuaddr(val),
            7 => r.write_ppudata(&mut self.bus, val),
            _ => {}
        }
    }

    pub fn advance_frame(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
    }

    fn render_scanline(&mut self) {
        match self.scanlines {
            0..240 => {
                let height = if self.regs.ctrl.contains(PPUCTRL::SPRITE_SIZE) {
                    16
                } else {
                    8
                };

                let sprite_buffer: Vec<Sprite> = (0..64usize)
                    .map(|i| self.oam.get_sprite(i))
                    .filter(|s| s.covers_scanline(self.scanlines, height))
                    .take(8)
                    .collect();

                self.render_background();

                sprite_buffer
                    .iter()
                    .rev()
                    .for_each(|s| self.render_sprite(s));
            }
            _ => {}
        }
    }

    fn render_sprite(&mut self, sprite: &Sprite) {
        let top = sprite.y as u16 + 1;
        let mut row = (self.scanlines - top) as u16;

        let height = if self.regs.ctrl.contains(PPUCTRL::SPRITE_SIZE) {
            16
        } else {
            8
        };

        if sprite.get_vertical_flip() {
            row = height as u16 - 1 - row;
        }
        let pallete_idx = sprite.get_palette();

        let palette = self.bus.palette_lookup(true, pallete_idx);

        let bank: u16 = self.regs.ctrl.get_sprite_addr();
        let base = (bank + (sprite.tile as u16 * 16)) as u16;

        let mut upper = self.bus.read(base + row);
        let mut lower = self.bus.read(base + row + 8);

        let buf_y = self.scanlines;
        for x in (0..=7).rev() {
            let buf_x = if sprite.get_horizontal_flip() {
                sprite.x + 7 - x
            } else {
                sprite.x + x
            };

            let color = (1 & lower) << 1 | (1 & upper);
            upper = upper >> 1;
            lower = lower >> 1;

            if color == 0 {
                continue;
            }

            let value = palette[color as usize];

            if buf_x < 8 && !self.regs.mask.contains(PPUMASK::SHOW_SPRITES) {
                continue;
            };

            if (buf_y as usize) < Y_SIZE && (buf_x as usize) < X_SIZE {
                if sprite.get_priority() && self.buffer[buf_y as usize][buf_x as usize] & 0x1 != 0 {
                    self.buffer[buf_y as usize][buf_x as usize] = value;
                }
            }
        }
    }

    fn render_background(&mut self) {
        if !self.regs.mask.contains(PPUMASK::BACKGROUND_RENDERING) {
            return;
        } else {
            for x in 0..256 {
                self.buffer[self.scanlines as usize][x] = self.bus.read(0x3F00);
            }
        }
        let pattern_base = self.regs.ctrl.get_bg_addr();
        let nametable = 0x2000 + (self.regs.ctrl.bits() as u16 & 0x03) * 0x400;
        let attributes = nametable + 0x03C0;

        let y = self.scanlines;
        let ty = y >> 3;
        let fine_y = y & 0x7;

        for tx in 0..32u16 {
            let tile = self.bus.read(nametable + ty * 32 + tx) as u16;
            let attr = self.bus.read(attributes + (ty >> 2) * 8 + (tx >> 2));
            let shift = ((ty & 2) << 1) | (tx & 2);
            let palette_idx = (attr >> shift) & 0x03;

            let palette = self.bus.palette_lookup(false, palette_idx);

            let base = pattern_base + tile * 16;

            let lo = self.bus.read(base + fine_y);
            let hi = self.bus.read(base + fine_y + 8);
            for fine_x in 0..8u16 {
                let bit = 7 - fine_x;
                let colour = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1);

                let value = palette[colour as usize];
                let x = tx * 8 + fine_x;
                if x < 8 && !self.regs.mask.contains(PPUMASK::SHOW_BACKGROUND) {
                    continue;
                };
                self.buffer[y as usize][x as usize] = value;
            }
        }
    }

    pub fn render(&self) -> FrameBuffer {
        self.buffer
    }
}
