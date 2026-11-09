use super::*;

const DOTS_PER_LINE: u16 = 341;
const SCANLINES: u16 = 262;

fn tick_to(ppu: &mut Ppu, scanline: u16, dot: u16) {
    let mut guard = 0u32;
    while !(ppu.scanlines as u16 == scanline && ppu.dots as u16 == dot) {
        ppu.tick();
        guard += 1;
        assert!(
            guard < DOTS_PER_LINE as u32 * SCANLINES as u32 * 2,
            "tick_to never reached ({scanline}, {dot})"
        );
    }
}

// --- NMI -------------------------------------------------------------------

#[test]
fn vblank_sets_at_241_1_and_clears_at_261_1() {
    let mut ppu = Ppu::new();

    tick_to(&mut ppu, 241, 0);
    assert_eq!(ppu.regs.status.bits() & 0x80, 0x00, "vblank before 241,1");

    ppu.tick();
    assert_eq!(
        ppu.regs.status.bits() & 0x80,
        0x80,
        "vblank not set at 241,1"
    );

    tick_to(&mut ppu, 261, 0);
    ppu.tick();
    assert_eq!(
        ppu.regs.status.bits() & 0x80,
        0x00,
        "vblank not cleared at 261,1"
    );
}

#[test]
fn nmi_latches_once_per_vblank() {
    let mut ppu = Ppu::new();
    ppu.reg_write(0, 0x80); // NMI enable

    tick_to(&mut ppu, 241, 0);
    assert!(!ppu.take_nmi(), "NMI before vblank starts");

    ppu.tick();
    assert!(ppu.take_nmi(), "no NMI on the vblank edge");
    assert!(!ppu.take_nmi(), "latch is not one-shot");
}

#[test]
fn nmi_not_raised_when_disabled() {
    let mut ppu = Ppu::new();
    // PPUCTRL bit 7 stays clear

    tick_to(&mut ppu, 241, 1);
    assert!(!ppu.take_nmi(), "NMI raised with NMI_ENABLE clear");
}

#[test]
fn enabling_nmi_during_vblank_raises_immediately() {
    let mut ppu = Ppu::new();

    tick_to(&mut ppu, 241, 1);
    assert!(!ppu.take_nmi());

    ppu.reg_write(0, 0x80); // 0 -> 1 while vblank is set
    assert!(ppu.take_nmi(), "enabling NMI during vblank must raise it");
    assert!(!ppu.take_nmi());
}

#[test]
fn reading_status_clears_vblank_but_not_a_latched_nmi() {
    let mut ppu = Ppu::new();
    ppu.reg_write(0, 0x80);
    tick_to(&mut ppu, 241, 1);

    let status = ppu.reg_read(2);
    assert_eq!(status & 0x80, 0x80, "vblank not reported before clearing");
    assert_eq!(ppu.regs.status.bits() & 0x80, 0x00, "vblank not cleared");

    assert!(ppu.take_nmi(), "reading $2002 cancelled a latched NMI");
}

#[test]
fn status_read_resets_write_toggle() {
    let mut ppu = Ppu::new();

    ppu.reg_write(6, 0x21); // first write: w = 1
    let _ = ppu.reg_read(2); // must reset w
    ppu.reg_write(6, 0x00); // should be a high-byte write again

    // $2006 high-byte write with 0 leaves the low 8 bits of v alone
    assert_eq!(ppu.regs.v & 0x00FF, 0x00);
}

// --- render ----------------------------------------------------------------

fn write_tile_row(ppu: &mut Ppu, table: u16, tile: u8, row: u8, lo: u8, hi: u8) {
    let base = table + tile as u16 * 16 + row as u16;
    ppu.bus.write(base, lo);
    ppu.bus.write(base + 8, hi);
}

fn setup_tile0(ppu: &mut Ppu, lo: u8, hi: u8) {
    ppu.reg_write(1, 0x08); // enable background
    ppu.bus.write(0x2000, 0x00); // nametable 0, tile 0
    ppu.bus.write(0x23C0, 0x00); // attribute: palette 0
    ppu.bus.write(0x3F00, 0x0F); // backdrop
    write_tile_row(ppu, 0x0000, 0, 0, lo, hi);
}

#[test]
fn whole_screen_is_backdrop_when_background_disabled() {
    let mut ppu = Ppu::new();
    ppu.bus.write(0x3F00, 0x16);
    // PPUMASK bit 3 stays clear

    let fb = ppu.render();

    for y in 0..240 {
        for x in 0..256 {
            assert_eq!(fb[y][x], 0x16, "at ({x}, {y})");
        }
    }
}

#[test]
fn low_bitplane_selects_colour_1() {
    let mut ppu = Ppu::new();
    setup_tile0(&mut ppu, 0b1000_0000, 0x00);
    ppu.bus.write(0x3F01, 0x21);

    let fb = ppu.render();

    assert_eq!(fb[0][0], 0x21);
    assert_eq!(fb[0][1], 0x0F, "colour 0 must fall back to the backdrop");
}

#[test]
fn high_bitplane_selects_colour_2() {
    let mut ppu = Ppu::new();
    setup_tile0(&mut ppu, 0x00, 0b1000_0000);
    ppu.bus.write(0x3F02, 0x22);

    let fb = ppu.render();

    assert_eq!(fb[0][0], 0x22);
}

#[test]
fn attribute_selects_palette_per_quadrant() {
    let mut ppu = Ppu::new();
    ppu.reg_write(1, 0x08);

    // one tile per quadrant of the 4x4-tile (32x32 px) attribute block
    for (tx, ty) in [(0u16, 0u16), (2, 0), (0, 2), (2, 2)] {
        ppu.bus.write(0x2000 + ty * 32 + tx, 0x00);
    }
    ppu.bus.write(0x23C0, 0b11_10_01_00);

    ppu.bus.write(0x3F00, 0x0F);
    ppu.bus.write(0x3F01, 0x11); // palette 0, colour 1
    ppu.bus.write(0x3F05, 0x12); // palette 1
    ppu.bus.write(0x3F09, 0x13); // palette 2
    ppu.bus.write(0x3F0D, 0x14); // palette 3

    write_tile_row(&mut ppu, 0x0000, 0, 0, 0xFF, 0x00);

    let fb = ppu.render();

    assert_eq!(fb[0][0], 0x11, "top-left quadrant");
    assert_eq!(fb[0][16], 0x12, "top-right quadrant");
    assert_eq!(fb[16][0], 0x13, "bottom-left quadrant");
    assert_eq!(fb[16][16], 0x14, "bottom-right quadrant");
}

#[test]
fn nametable_base_follows_ppuctrl_bits() {
    let mut ppu = Ppu::new();
    ppu.reg_write(1, 0x08);
    ppu.reg_write(0, 0x01); // nametable 1 -> $2400

    ppu.bus.write(0x2400, 0x00);
    ppu.bus.write(0x27C0, 0x00); // attribute table for $2400
    ppu.bus.write(0x3F00, 0x0F);
    ppu.bus.write(0x3F01, 0x31);
    write_tile_row(&mut ppu, 0x0000, 0, 0, 0xFF, 0x00);

    let fb = ppu.render();

    assert_eq!(fb[0][0], 0x31);
}

#[test]
fn pattern_table_follows_ppuctrl_bit_4() {
    let mut ppu = Ppu::new();
    ppu.reg_write(1, 0x08);
    ppu.reg_write(0, 0x10); // pattern table 1 -> $1000

    ppu.bus.write(0x2000, 0x00);
    ppu.bus.write(0x23C0, 0x00);
    ppu.bus.write(0x3F00, 0x0F);
    ppu.bus.write(0x3F01, 0x41);
    write_tile_row(&mut ppu, 0x1000, 0, 0, 0xFF, 0x00);

    let fb = ppu.render();

    assert_eq!(fb[0][0], 0x41);
}

#[test]
fn tile_index_comes_from_the_nametable() {
    let mut ppu = Ppu::new();
    ppu.reg_write(1, 0x08);

    // tile 7 in the nametable, pattern only defined for tile 7
    ppu.bus.write(0x2000, 0x07);
    ppu.bus.write(0x23C0, 0x00);
    ppu.bus.write(0x3F00, 0x0F);
    ppu.bus.write(0x3F01, 0x51);
    write_tile_row(&mut ppu, 0x0000, 7, 0, 0xFF, 0x00);

    let fb = ppu.render();

    assert_eq!(fb[0][0], 0x51);
}

#[test]
fn tile_row_is_selected_by_pixel_y() {
    let mut ppu = Ppu::new();
    ppu.reg_write(1, 0x08);

    ppu.bus.write(0x2000, 0x00);
    ppu.bus.write(0x23C0, 0x00);
    ppu.bus.write(0x3F00, 0x0F);
    ppu.bus.write(0x3F01, 0x61);

    // row 0 is empty, row 3 has a colour-1 pixel on the left
    write_tile_row(&mut ppu, 0x0000, 0, 0, 0x00, 0x00);
    write_tile_row(&mut ppu, 0x0000, 0, 3, 0b1000_0000, 0x00);

    let fb = ppu.render();

    assert_eq!(fb[0][0], 0x0F, "row 0 should be backdrop");
    assert_eq!(fb[3][0], 0x61, "row 3 should use the pattern row");
}
