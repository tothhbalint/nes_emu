use crate::ppu::{
    bus::PpuBus,
    oam::{Sprite, SPRITE_COUNT},
    registers::{Registers, PPUMASK},
    FrameBuffer, X_SIZE, Y_SIZE,
};

pub fn render(
    sprites: &[Sprite; SPRITE_COUNT],
    mut mask: u64,
    regs: &Registers,
    bus: &PpuBus,
    buffer: &mut FrameBuffer,
    y: u16,
    height: u8,
) -> bool {
    let mut sprite0_hit = false;

    while mask != 0 {
        let i = 63 - mask.leading_zeros() as usize;
        mask &= !(1u64 << i);

        sprite0_hit |= render_sprite(i, &sprites[i], regs, bus, buffer, y, height);
    }

    sprite0_hit
}

fn render_sprite(
    index: usize,
    sprite: &Sprite,
    regs: &Registers,
    bus: &PpuBus,
    buffer: &mut FrameBuffer,
    y: u16,
    height: u8,
) -> bool {
    let top = sprite.y as u16 + 1;
    let mut row = (y - top) as u16;
    if sprite.get_vertical_flip() {
        row = height as u16 - 1 - row;
    }
    let mut hit = false;
    let pallete_idx = sprite.get_palette();

    let palette = bus.palette_lookup(true, pallete_idx);

    let bank: u16 = regs.ctrl.get_sprite_addr();
    let base = (bank + (sprite.tile as u16 * 16)) as u16;

    let mut upper = bus.read(base + row);
    let mut lower = bus.read(base + row + 8);
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

        if buf_x < 8 && !regs.mask.contains(PPUMASK::SHOW_SPRITES) {
            continue;
        };

        if (y as usize) < Y_SIZE && (buf_x as usize) < X_SIZE {
            buffer[y as usize][buf_x as usize] = value;
        }
        if index == 0
            && color != 0
            && buffer[y as usize][buf_x as usize] != 0
            && buf_x != 255
            && (buf_x >= 8)
        {
            hit = true;
        }
    }
    return hit;
}
