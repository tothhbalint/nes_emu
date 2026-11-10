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
        const SPRITE_OVERFLOW = 1 << 5;
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

    pub read_buffer: u8,
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            ctrl: PPUCTRL::empty(),
            mask: PPUMASK::empty(),
            status: PPUSTATUS::empty(),
            v: 0,
            t: 0,
            x: 0,
            w: false,
            read_buffer: 0,
        }
    }
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
        let addr = self.v & 0x3FFF;

        let data = if addr >= 0x3F00 {
            let value = bus.read(addr);
            self.read_buffer = bus.read(addr & 0x2FFF);
            value
        } else {
            let value = self.read_buffer;
            self.read_buffer = bus.read(addr);
            value
        };

        self.v = self.v.wrapping_add(self.increment() as u16) & 0x7FFF;
        data
    }

    pub fn write_ppudata(&mut self, bus: &mut PpuBus, val: u8) {
        {
            bus.write(self.v & 0x3FFF, val);
            self.v = self.v.wrapping_add(self.increment() as u16) & 0x7FFF;
        }
    }

    fn y_increment(&mut self) {
        let v = &mut self.v;
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
    fn increment_x(&mut self) {
        let v = &mut self.v;
        if *v & 0x001F == 0x001F {
            *v &= !0x001F;
            *v ^= 0x0400;
        } else {
            *v += 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================
    // PPUCTRL ($2000)
    // ==========================================

    #[test]
    fn write_ctrl_sets_register() {
        let mut regs = Registers::default();

        regs.write_ctrl(0x80);

        assert_eq!(regs.read_ctrl(), 0x80);
    }

    #[test]
    fn write_ctrl_sets_nametable_bits_in_t() {
        let mut regs = Registers::default();

        regs.write_ctrl(0b0000_0011);

        assert_eq!(regs.t & 0x0C00, 0x0C00);
    }

    #[test]
    fn write_ctrl_preserves_other_bits_in_t() {
        let mut regs = Registers::default();
        regs.t = 0x73FF;

        regs.write_ctrl(0x02);

        assert_eq!(regs.t & !0x0C00, 0x73FF & !0x0C00);
    }

    // ==========================================
    // PPUMASK ($2001)
    // ==========================================

    #[test]
    fn write_mask_sets_register() {
        let mut regs = Registers::default();

        regs.write_mask(0x1E);

        assert_eq!(regs.read_mask(), 0x1E);
    }

    // ==========================================
    // PPUSTATUS ($2002)
    // ==========================================

    #[test]
    fn read_status_returns_status() {
        let mut regs = Registers::default();
        regs.status = PPUSTATUS::VBLANK;

        assert_eq!(regs.read_status(), 0x80);
    }

    #[test]
    fn read_status_clears_vblank() {
        let mut regs = Registers::default();
        regs.status = PPUSTATUS::VBLANK;

        regs.read_status();

        assert!(!regs.status.contains(PPUSTATUS::VBLANK));
    }

    #[test]
    fn read_status_resets_write_toggle() {
        let mut regs = Registers::default();

        regs.write_ppuscroll(0x12);
        assert!(regs.w);

        regs.read_status();

        assert!(!regs.w);
    }

    #[test]
    fn read_status_preserves_other_flags() {
        let mut regs = Registers::default();
        regs.status = PPUSTATUS::SPRITE_0 | PPUSTATUS::SPRITE_OVERFLOW | PPUSTATUS::VBLANK;

        regs.read_status();

        assert!(regs.status.contains(PPUSTATUS::SPRITE_0));
        assert!(regs.status.contains(PPUSTATUS::SPRITE_OVERFLOW));
        assert!(!regs.status.contains(PPUSTATUS::VBLANK));
    }

    // ==========================================
    // PPUADDR ($2006)
    // ==========================================

    #[test]
    fn ppuaddr_first_write_sets_high_byte() {
        let mut regs = Registers::default();

        regs.write_ppuaddr(0x23);

        assert_eq!(regs.t & 0x7F00, 0x2300);
        assert!(regs.w);
    }

    #[test]
    fn ppuaddr_first_write_masks_to_six_bits() {
        let mut regs = Registers::default();

        regs.write_ppuaddr(0xFF);

        assert_eq!(regs.t & 0x7F00, 0x3F00);
    }

    #[test]
    fn ppuaddr_second_write_sets_low_byte() {
        let mut regs = Registers::default();

        regs.write_ppuaddr(0x23);
        regs.write_ppuaddr(0x45);

        assert_eq!(regs.v, 0x2345);
        assert_eq!(regs.t, 0x2345);
    }

    #[test]
    fn ppuaddr_second_write_resets_toggle() {
        let mut regs = Registers::default();

        regs.write_ppuaddr(0x23);
        regs.write_ppuaddr(0x45);

        assert!(!regs.w);
    }

    #[test]
    fn ppuaddr_first_write_preserves_low_byte() {
        let mut regs = Registers::default();
        regs.t = 0x0045;

        regs.write_ppuaddr(0x23);

        assert_eq!(regs.t, 0x2345);
    }

    // ==========================================
    // PPUSCROLL ($2005)
    // ==========================================

    #[test]
    fn scroll_first_write_sets_coarse_x() {
        let mut regs = Registers::default();

        regs.write_ppuscroll(0b0010_1011);

        assert_eq!(regs.t & 0x001F, 5);
    }

    #[test]
    fn scroll_first_write_sets_fine_x() {
        let mut regs = Registers::default();

        regs.write_ppuscroll(0b0010_1011);

        assert_eq!(regs.x, 3);
    }

    #[test]
    fn scroll_first_write_sets_toggle() {
        let mut regs = Registers::default();

        regs.write_ppuscroll(0x00);

        assert!(regs.w);
    }

    #[test]
    fn scroll_second_write_sets_coarse_y() {
        let mut regs = Registers::default();

        regs.write_ppuscroll(0x00);
        regs.write_ppuscroll(0b1010_1011);

        assert_eq!((regs.t >> 5) & 0x1F, 21);
    }

    #[test]
    fn scroll_second_write_sets_fine_y() {
        let mut regs = Registers::default();

        regs.write_ppuscroll(0x00);
        regs.write_ppuscroll(0b0000_0110);

        assert_eq!((regs.t >> 12) & 0x07, 6);
    }

    #[test]
    fn scroll_second_write_resets_toggle() {
        let mut regs = Registers::default();

        regs.write_ppuscroll(0x00);
        regs.write_ppuscroll(0x00);

        assert!(!regs.w);
    }

    // ==========================================
    // Increment X
    // ==========================================

    #[test]
    fn increment_x_increments_coarse_x() {
        let mut regs = Registers::default();
        regs.v = 0x0005;

        regs.increment_x();

        assert_eq!(regs.v, 0x0006);
    }

    #[test]
    fn increment_x_wraps_coarse_x() {
        let mut regs = Registers::default();
        regs.v = 0x001F;

        regs.increment_x();

        assert_eq!(regs.v, 0x0400);
    }

    #[test]
    fn increment_x_toggles_horizontal_nametable() {
        let mut regs = Registers::default();
        regs.v = 0x041F;

        regs.increment_x();

        assert_eq!(regs.v, 0x0000);
    }

    #[test]
    fn increment_x_preserves_vertical_bits() {
        let mut regs = Registers::default();
        regs.v = 0x7BE5;

        regs.increment_x();

        assert_eq!(regs.v, 0x7BE6);
    }

    // ==========================================
    // Increment Y
    // ==========================================

    #[test]
    fn increment_y_increments_fine_y() {
        let mut regs = Registers::default();
        regs.v = 0x0020;

        regs.y_increment();

        assert_eq!(regs.v, 0x1020);
    }

    #[test]
    fn increment_y_resets_fine_y_and_increments_coarse_y() {
        let mut regs = Registers::default();
        regs.v = 0x7000;

        regs.y_increment();

        assert_eq!(regs.v, 0x0020);
    }

    #[test]
    fn increment_y_wraps_coarse_y_at_29() {
        let mut regs = Registers::default();
        regs.v = 0x7000 | (29 << 5);

        regs.y_increment();

        assert_eq!(regs.v, 0x0800);
    }

    #[test]
    fn increment_y_wraps_coarse_y_at_31() {
        let mut regs = Registers::default();
        regs.v = 0x7000 | (31 << 5);

        regs.y_increment();

        assert_eq!(regs.v, 0x0000);
    }

    #[test]
    fn increment_y_preserves_horizontal_bits() {
        let mut regs = Registers::default();
        regs.v = 0x041F;

        regs.y_increment();

        assert_eq!(regs.v, 0x141F);
    }

    // ==========================================
    // VRAM address increment
    // ==========================================

    #[test]
    fn increment_is_one_by_default() {
        let regs = Registers::default();

        assert_eq!(regs.increment(), 1);
    }

    #[test]
    fn increment_is_32_when_ctrl_bit_is_set() {
        let mut regs = Registers::default();

        regs.write_ctrl(0x04);

        assert_eq!(regs.increment(), 32);
    }
}
