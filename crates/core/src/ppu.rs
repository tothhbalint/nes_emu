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

                read_buffer: 0,
            },
            bus: PpuBus::new(),
            oam: Oam::new(),
            nmi_line: false,
            nmi_latch: false,
            scroll_x: 0,
            scroll_y: 0,

            dots: 0,
            scanlines: 261,
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

        let rendering_enabled = self
            .regs
            .mask
            .intersects(PPUMASK::BACKGROUND_RENDERING | PPUMASK::SPRITE_RENDERING);

        match (self.scanlines, self.dots) {
            (0..=239, 1) => self.render_scanline(),

            (241, 1) => {
                self.regs.status.insert(PPUSTATUS::VBLANK);
                self.sample_nmi_line();
            }

            (261, 1) => {
                self.regs.status.remove(PPUSTATUS::VBLANK);
                self.regs.status.remove(PPUSTATUS::SPRITE_0);
                self.sample_nmi_line();
            }

            (0..=239, 256) | (261, 256) if rendering_enabled => {
                self.y_increment();
            }

            (0..=239, 257) | (261, 257) if rendering_enabled => {
                self.regs.v = (self.regs.v & !0x041F) | (self.regs.t & 0x041F);
            }

            (261, 280..=304) if rendering_enabled => {
                self.regs.v = (self.regs.v & !0x7BE0) | (self.regs.t & 0x7BE0);
            }

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
            2 => {
                let value = r.read_status();
                self.sample_nmi_line();
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

    fn y_increment(&mut self) {
        let v = &mut self.regs.v;
        if *v & 0x7000 != 0x7000 {
            *v += 0x1000;
            return;
        }
        *v &= !0x7000;
        let mut y = (*v & 0x03E0) >> 5;

        if y == 29 {
            y = 0;
            *v ^= 0x0800;
        } else if y == 31 {
            y = 0
        } else {
            y += 1;
        }
        *v = (*v & !0x03E0) | (y << 5)
    }

    #[inline]
    fn inc_coarse_x(&mut self) {
        if self.regs.v & 0x001F == 0x001F {
            self.regs.v &= !0x001F;
            self.regs.v ^= 0x0400;
        } else {
            self.regs.v += 1;
        }
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
                self.buffer[buf_y as usize][buf_x as usize] = value;
            }
        }
    }

    fn render_background(&mut self) {
        let backdrop = self.bus.read(0x3F00);
        self.buffer[self.scanlines as usize].fill(backdrop);

        if !self.regs.mask.contains(PPUMASK::BACKGROUND_RENDERING) {
            return;
        }

        let pattern_base = self.regs.ctrl.get_bg_addr();
        let fine_x = self.regs.x as u16;
        let y = self.scanlines as u16;

        let mut v = self.regs.v;

        for tile_i in 0..=32u16 {
            let nt_addr = 0x2000 | (v & 0x0FFF);
            let attr_addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);

            let tile = self.bus.read(nt_addr) as u16;
            let attr = self.bus.read(attr_addr);

            let shift = ((v >> 4) & 4) | (v & 2);

            let palette = self.bus.palette_lookup(false, (attr >> shift) & 0x03);
            let fine_y = (v >> 12) & 0x07;
            let base = pattern_base + tile * 16;

            let lo = self.bus.read(base + fine_y);
            let hi = self.bus.read(base + fine_y + 8);

            if self.scanlines == 0 {
                if y == 0 && tile_i < 8 {
                    println!(
                        "tile_i={} v={:04X} nt={:04X} tile={:02X} pattern={:04X}",
                        tile_i, v, nt_addr, tile, base
                    );
                }
            }
            for px in 0..8u16 {
                let source_x = tile_i * 8 + px;
                if source_x < fine_x {
                    continue; // scrolled off
                }
                let x = source_x - fine_x;
                if x >= 256 {
                    break;
                }
                let bit = 7 - px;
                let colour = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1);
                if colour == 0 {
                    continue;
                }
                if x < 8 && !self.regs.mask.contains(PPUMASK::SHOW_BACKGROUND) {
                    continue;
                };
                self.buffer[y as usize][x as usize] = palette[colour as usize];
            }
            if v & 0x001F == 0x001F {
                v &= !0x001F;
                v ^= 0x0400;
            } else {
                v += 1;
            }
        }
    }

    pub fn render(&self) -> FrameBuffer {
        self.buffer
    }
}
