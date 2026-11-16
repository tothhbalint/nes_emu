use crate::ppu::registers::{Registers, PPUMASK};
use crate::ppu::{bus::PpuBus, FrameBuffer};

pub fn render(regs: &Registers, y: u16, bus: &PpuBus, buffer: &mut FrameBuffer) {
    let backdrop = bus.read(0x3F00);
    buffer[y as usize].fill(backdrop);

    if !regs.mask.contains(PPUMASK::BACKGROUND_RENDERING) {
        return;
    }

    let pattern_base = regs.ctrl.get_bg_addr();
    let fine_x = regs.x as u16;

    let mut v = regs.v;

    for tile_i in 0..=32u16 {
        let nt_addr = 0x2000 | (v & 0x0FFF);
        let attr_addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);

        let tile = bus.read(nt_addr) as u16;
        let attr = bus.read(attr_addr);

        let shift = ((v >> 4) & 4) | (v & 2);

        let palette = bus.palette_lookup(false, (attr >> shift) & 0x03);
        let fine_y = (v >> 12) & 0x07;
        let base = pattern_base + tile * 16;

        let lo = bus.read(base + fine_y);
        let hi = bus.read(base + fine_y + 8);

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
            if x < 8 && !regs.mask.contains(PPUMASK::SHOW_BACKGROUND) {
                continue;
            };
            buffer[y as usize][x as usize] = palette[colour as usize];
        }
        if v & 0x001F == 0x001F {
            v &= !0x001F;
            v ^= 0x0400;
        } else {
            v += 1;
        }
    }
}
