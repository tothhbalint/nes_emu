use bitflags::bitflags;

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
    pub fn write_ctrl(&mut self, val: u8) {}

    pub fn write_mask(&mut self, val: u8) {}
}
