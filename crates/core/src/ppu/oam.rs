#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpriteByte {
    Y = 0,
    Tile = 1,
    Attr = 2,
    X = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct Sprite {
    pub y: u8,
    pub tile: u8,
    pub attr: u8,
    pub x: u8,
}

impl Sprite {
    pub fn byte(&self, index: u8) -> u8 {
        match index {
            0 => self.y,
            1 => self.tile,
            2 => self.attr,
            3 => self.x,
            _ => panic!(),
        }
    }

    pub fn set_byte(&mut self, index: u8, val: u8) {
        match index {
            0 => self.y = val,
            1 => self.tile = val,
            2 => self.attr = val,
            3 => self.x = val,
            _ => panic!(),
        }
    }
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
pub(crate) const SPRITE_COUNT: usize = 64;
const OAM_SIZE: usize = SPRITE_COUNT * SPRITE_SIZE;

pub struct Oam {
    addr: u8,
    data: [Sprite; SPRITE_COUNT],
}

impl Default for Oam {
    fn default() -> Self {
        Oam {
            addr: 0,
            data: [Sprite::default(); SPRITE_COUNT],
        }
    }
}

impl Oam {
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
        self.data[(self.addr >> 2) as usize].byte(self.addr & 0x3)
    }

    #[inline]
    pub fn write(&mut self, val: u8) {
        self.data[(self.addr >> 2) as usize].set_byte(self.addr & 0x3, val);
        self.addr = self.addr.wrapping_add(1);
    }

    #[inline]
    pub fn sprites(&self) -> &[Sprite; SPRITE_COUNT] {
        &self.data
    }

    #[inline]
    pub fn get_sprite(&self, index: usize) -> Sprite {
        self.data[index]
    }

    pub fn select(&self, scanline: u16, height: u8) -> (u64, bool) {
        let mut selected: u64 = 0;
        let mut count: u8 = 0;
        let mut overflow = false;

        for (i, sprite) in self.data.iter().enumerate() {
            if !sprite.covers_scanline(scanline, height) {
                continue;
            }

            if count < 8 {
                selected |= 1u64 << i;
                count += 1;
            } else {
                overflow = true;
                break;
            }
        }
        (selected, overflow)
    }
}
