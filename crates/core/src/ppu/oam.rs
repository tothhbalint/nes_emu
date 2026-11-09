#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpriteByte {
    Y = 0,
    Tile = 1,
    Attr = 2,
    X = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Sprite {
    pub y: u8,
    pub tile: u8,
    pub attr: u8,
    pub x: u8,
}
/*
 * 76543210
 * ||||||||
 * ||||||++- Palette (4 to 7) of sprite
 * |||+++--- Unimplemented (read 0)
 * ||+------ Priority (0: in front of background; 1: behind background)
 * |+------- Flip sprite horizontally
 * +-------- Flip sprite vertically
 *
 */
impl Sprite {
    pub fn get_palette(&self) -> u8 {
        self.attr & 0x3
    }

    pub fn get_priority(&self) -> bool {
        (self.attr >> 5 & 0x1) == 0
    }

    pub fn get_horizontal_flip(&self) -> bool {
        (self.attr >> 6 & 0x1) == 1
    }

    pub fn get_vertical_flip(&self) -> bool {
        (self.attr >> 7 & 0x1) == 1
    }

    pub fn covers_scanline(&self, scanline: u16, height: u8) -> bool {
        let top = self.y as u16 + 1;
        scanline >= top && scanline < top + height as u16
    }
}

const SPRITE_SIZE: usize = 4;
const SPRITE_COUNT: usize = 64;
const OAM_SIZE: usize = SPRITE_COUNT * SPRITE_SIZE;

pub struct Oam {
    pub addr: u8,
    pub data: [u8; OAM_SIZE],
}

impl Oam {
    pub const fn new() -> Self {
        Oam {
            addr: 0,
            data: [0; OAM_SIZE],
        }
    }

    #[inline]
    pub fn addr(&self) -> u8 {
        self.addr
    }

    #[inline]
    pub fn set_addr(&mut self, addr: u8) {
        self.addr = addr;
    }

    #[inline]
    pub fn peek(&self) -> u8 {
        self.data[self.addr as usize]
    }

    #[inline]
    pub fn write(&mut self, val: u8) {
        self.data[self.addr as usize] = val;
        self.addr = self.addr.wrapping_add(1);
    }

    #[inline]
    pub fn get_sprite(&self, index: usize) -> Sprite {
        let base = (index & (SPRITE_COUNT - 1)) * SPRITE_SIZE;

        Sprite {
            y: (self.data[base + SpriteByte::Y as usize]),
            tile: (self.data[base + SpriteByte::Tile as usize]),
            attr: (self.data[base + SpriteByte::Attr as usize]),
            x: (self.data[base + SpriteByte::X as usize]),
        }
    }
}
