use bitflags::bitflags;

use crate::ppu::bus::PpuBus;

/*
* 7  bit  0
---- ----
VPHB SINN
|||| ||||
|||| ||++- Base nametable address
|||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
|||| |+--- VRAM address increment per CPU read/write of PPUDATA
|||| |     (0: add 1, going across; 1: add 32, going down)
|||| +---- Sprite pattern table address for 8x8 sprites
||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
|||+------ Background pattern table address (0: $0000; 1: $1000)
||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
|+-------- PPU master/slave select
|          (0: read backdrop from EXT pins; 1: output color on EXT pins)
+--------- Vblank NMI enable (0: off, 1: on)
*/

bitflags! {
    #[derive(Clone, Copy)]
    pub struct PPUCTRL: u8 {
        const NAMETABLE_1 = 1 << 0;
        const NAMETABLE_2 = 1 << 1;
        const VRAM_ADDR_INCREMENT = 1 << 2;
        const SPRITE_PATTERN_ADDRESS = 1 << 3;
        const BACKGROUND_PATTERN_ADDRESS = 1 << 4;
        const SPRITE_SIZE = 1 << 5;
        const MASTER_SLAVE_SELECT = 1 << 6;
        const NMI_ENABLE = 1 << 7;
    }
}

impl PPUCTRL {
    pub fn get_bg_addr(&self) -> u16 {
        if self.contains(PPUCTRL::BACKGROUND_PATTERN_ADDRESS) {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn get_sprite_addr(&self) -> u16 {
        if self.contains(PPUCTRL::SPRITE_PATTERN_ADDRESS) {
            0x1000
        } else {
            0x0000
        }
    }
}

/*
* 7  bit  0
---- ----
BGRs bMmG
|||| ||||
|||| |||+- Greyscale (0: normal color, 1: greyscale)
|||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
|||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
|||| +---- 1: Enable background rendering
|||+------ 1: Enable sprite rendering
||+------- Emphasize red (green on PAL/Dendy)
|+-------- Emphasize green (red on PAL/Dendy)
+--------- Emphasize blue
*/

bitflags! {
    #[derive(Clone, Copy)]
    pub struct PPUMASK: u8 {
        const GRAYSCALE = 1 << 0;
        const SHOW_BACKGROUND = 1 << 1;
        const SHOW_SPRITES = 1 << 2;
        const BACKGROUND_RENDERING = 1 << 3;
        const SPRITE_RENDERING = 1 << 4;
        const EMPH_RED = 1 << 5;
        const EMPH_GREEN = 1 << 6;
        const EMPH_BLUE = 1 << 7;
    }
}
/*

7  bit  0
---- ----
VSOx xxxx
|||| ||||
|||+-++++- (PPU open bus or 2C05 PPU identifier)
||+------- Sprite overflow flag
|+-------- Sprite 0 hit flag
+--------- Vblank flag, cleared on read. Unreliable; see below.

*/
bitflags! {
    #[derive(Clone, Copy)]
    pub struct PPUSTATUS: u8 {
        const SPITE_OVERFLOW = 1 << 5;
        const SPRITE_0 = 1 << 6;
        const VBLANK = 1 <<7;
    }
}

pub struct Registers {
    pub ctrl: PPUCTRL,
    pub mask: PPUMASK,
    pub status: PPUSTATUS,

    pub v: u16,  // VRAM
    pub t: u16,  // TEMP VRAM
    pub x: u8,   // 3-bit fine X
    pub w: bool, // $2005/2006 write toggle
}

impl Registers {
    pub fn write_ctrl(&mut self, val: u8) {
        self.ctrl = PPUCTRL::from_bits_retain(val);

        self.t = (self.t & 0xF3FF) | ((val as u16 & 0x03) << 10);
    }

    pub fn write_mask(&mut self, val: u8) {
        self.mask = PPUMASK::from_bits_retain(val);
    }

    pub fn write_status(&mut self, val: u8) {
        self.status = PPUSTATUS::from_bits_retain(val);
    }
    pub fn read_ctrl(&mut self) -> u8 {
        self.ctrl.bits()
    }

    pub fn read_mask(&mut self) -> u8 {
        self.mask.bits()
    }

    pub fn read_status(&mut self) -> u8 {
        self.w = false;
        let state = self.status.bits();
        self.status.remove(PPUSTATUS::VBLANK);

        state
    }

    pub fn write_ppuaddr(&mut self, val: u8) {
        if !self.w {
            self.t = (self.t & 0x00FF) | ((val as u16 & 0x3F) << 8);
        } else {
            self.t = (self.t & 0xFF00) | val as u16;
            self.v = self.t;
        }
        self.w = !self.w;
    }

    pub fn write_ppuscroll(&mut self, val: u8) {
        if !self.w {
            self.t = (self.t & 0x7FE0) | (val as u16 >> 3);
            self.x = val & 0x07;
        } else {
            self.t =
                (self.t & 0x0C1F) | (((val as u16) & 0x07) << 12) | (((val as u16) & 0xF8) << 2);
        }
        self.w = !self.w;
    }

    fn increment(&self) -> u8 {
        if self.ctrl.contains(PPUCTRL::VRAM_ADDR_INCREMENT) {
            32
        } else {
            1
        }
    }

    pub fn read_ppudata(&mut self, bus: &PpuBus) -> u8 {
        let data = bus.read(self.v);
        self.v = self.v.wrapping_add(self.increment() as u16) & 0x7FFF;
        data
    }

    pub fn write_ppudata(&mut self, bus: &mut PpuBus, val: u8) {
        {
            bus.write(self.v, val);
            self.v = self.v.wrapping_add(self.increment() as u16) & 0x7FFF;
        }
    }
}
