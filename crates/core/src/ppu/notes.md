# PPU Rendering Flow

How the NES PPU turns memory into a picture, and how to build it incrementally.

---

## 1. The model

The PPU is not a framebuffer renderer. It generates a video signal in real time, in
lockstep with the CRT beam; the emulator's framebuffer is a convenience so a host
window has something rectangular to display.

Rendering is **two independent layers composited per pixel**:

- **Background** — a 32×30 grid of 8×8 tiles.
- **Sprites** — up to 64 objects, at most 8 visible per scanline.

For every visible pixel the PPU decides which layer wins, maps the result through a
4-colour palette, and emits an index into a 64-entry system palette.

---

## 2. Timing: dots, scanlines, frames

| Unit | Definition | NTSC | PAL |
|---|---|---|---|
| Dot | one PPU clock (~186 ns) | — | — |
| Scanline | 341 dots | — | — |
| Frame | 262 scanlines | 262 | 312 |
| Visible lines | 0–239 | 240 | 240 |

`341 × 262 = 89,342` dots per frame at 5.369 MHz → 60.0988 Hz.

### Scanline layout

| Dots | Activity |
|---|---|
| 0 | idle |
| 1–256 | emit pixels; background fetch cycle runs in parallel |
| 257 | copy horizontal bits of `t` into `v` |
| 258–320 | sprite fetches for the next line |
| 321–336 | prefetch the first two background tiles of the next line |
| 337–340 | two dummy nametable fetches |

### Frame layout (NTSC)

| Scanlines | Purpose |
|---|---|
| 0–239 | visible picture |
| 240 | post-render, idle |
| 241–260 | vblank — the CPU's VRAM window |
| 261 | pre-render: `v ← t` vertical copy, flags cleared |

Rendering (fetch, sprite evaluation, pixel output) also runs on pre-render line 261,
which is what makes the first visible line of the next frame correct. Treat 261 as
"visible for timing, invisible for output".

**Odd-frame skip:** with rendering enabled, the pre-render line of every odd frame is
340 dots instead of 341. One-line special case; affects timing tests only.

---

## 3. Memory: where pixels come from

The PPU has its own 14-bit address space. The CPU reaches it only through `$2006`/`$2007`.

| PPU address | Contents |
|---|---|
| `$0000–$1FFF` | pattern tables (CHR ROM/RAM): 2 × 256 tiles |
| `$2000–$2EFF` | nametables + attribute tables (4 logical, 2 KB physical) |
| `$3000–$3EFF` | mirror of `$2000–$2EFF` |
| `$3F00–$3F1F` | palette RAM (32 bytes) |
| `$3F20–$3FFF` | mirror of palette RAM |

### Pattern table (per tile, 16 bytes)

```
byte  0..7   bitplane 0, one byte per row
byte  8..15  bitplane 1, one byte per row
```

Combining bit *n* of both planes gives a 2-bit colour index:

```
colour = (hi_bit << 1) | lo_bit
```

### Nametable (1 KB)

```
$2000 + 0x000..0x3BF   960 bytes: tile index, 32 per row, 30 rows
$2000 + 0x3C0..0x3FF    64 bytes: attribute table, 8x8 bytes
```

Each attribute byte covers a 16×16-pixel block (4×4 tiles) with four 2-bit palette
indices, one per quadrant:

```rust
let index  = (tile_y / 4) * 8 + (tile_x / 4);
let shift  = ((tile_y & 2) << 1) | (tile_x & 2); // 0, 2, 4, 6
let palette = (attr[index] >> shift) & 0b11;
```

### Palette RAM

Eight 4-colour palettes: 4 background (`$3F00–$3F0F`), 4 sprite (`$3F10–$3F1F`).
Entry 0 of each background palette is the universal backdrop; entry 0 of each sprite
palette means transparent.

`$3F10/$3F14/$3F18/$3F1C` alias `$3F00/$3F04/$3F08/$3F0C`.

---

## 4. Background pipeline

The PPU fetches **ahead** — one tile per 8 dots — and shifts the data out while the
next tile is being fetched.

### Fetch cycle (repeats every 8 dots)

| Dot offset | Fetch |
|---|---|
| 0–1 | nametable byte |
| 2–3 | attribute byte |
| 4–5 | pattern low byte |
| 6–7 | pattern high byte |

Runs during dots 1–256 (current line) and 321–336 (prefetch for the next).

### Shift registers

Four 16-bit registers — pattern low, pattern high, attribute low, attribute high —
shift left once per dot:

```rust
let bit = 15 - fine_x;
let lo = (bg_shift_lo >> bit) & 1;
let hi = (bg_shift_hi >> bit) & 1;
let colour = (hi << 1) | lo;
```

`fine_x` (3 bits from `PPUSCROLL`) selects which of the eight tile bits appears first,
which is how sub-tile horizontal scrolling works.

At the end of each 8-dot cycle the fetched tile loads into the shifter; the previously
loaded tile shifts into the high half.

---

## 5. The loopy registers

| Register | Width | Meaning |
|---|---|---|
| `v` | 15 bits | current VRAM address: `yyy NN YYYYY XXXXX` |
| `t` | 15 bits | temporary VRAM address (scroll latch) |
| `x` | 3 bits | fine X |
| `w` | 1 bit | `$2005`/`$2006` write toggle |

Bit layout of `v` / `t`:

```
14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
 |  |  |  +--+  +-----+  +--------+  +-----+
 |  |  |     |      |         |          |
 |  |  |     |      |         |          +-- coarse X (0-31)
 |  |  |     |      |         +------------- coarse Y (0-29)
 |  |  |     |      +----------------------- fine Y (0-7)
 |  |  +-----+------------------------------ nametable select
 +--+--------------------------------------- unused (always 0)
```

### When things move

| Event | Effect |
|---|---|
| `$2005` write #1 (`w=0`) | `t` coarse X + fine X ← value; `w=1` |
| `$2005` write #2 (`w=1`) | `t` coarse Y + fine Y ← value; `w=0` |
| `$2006` write #1 (`w=0`) | bits 8–14 of `t` ← value; `w=1` |
| `$2006` write #2 (`w=1`) | bits 0–7 of `t` ← value; `v ← t`; `w=0` |
| `$2007` read/write | access at `v`; `v += 1` or `32` per `PPUCTRL` bit 2 |
| `$2002` read | `w ← 0` |
| dot 256 of visible lines | increment coarse Y |
| dot 257 | `v` horizontal bits ← `t` |
| dots 280–304 of pre-render | `v` vertical bits ← `t` |

The copies at 257 and 280–304 are why scrolling works: `t` holds where the next frame
should start.

---

## 6. Sprites

OAM: 64 entries × 4 bytes.

| Byte | Field |
|---|---|
| 0 | Y position of the top edge, minus 1 (`$FF` hides) |
| 1 | tile index (8×16 mode: bit 0 selects pattern table) |
| 2 | palette (0–1), priority (5), flip H (6), flip V (7) |
| 3 | X position |

### Per scanline

1. **Evaluation** (dots 65–256): scan primary OAM for sprites whose Y range covers the
   next line, up to 8, into secondary OAM. A 9th candidate sets the overflow flag.
2. **Fetch** (dots 257–320): for those 8, fetch pattern bytes and attributes.
3. **Output** (dots 1–256): if `dot - 1` lies in a sprite's 8-pixel span, extract the
   pixel, apply horizontal flip, produce a colour index. Colour 0 is transparent.

### Priority

- Among sprites: lower OAM index wins.
- Sprite vs background: attribute bit 5 set means "behind background".
- Sprite 0 hit: set when sprite 0's opaque pixel overlaps an opaque background pixel,
  during dots 1–255, with both layers enabled and not clipped by the left-edge mask.

---

## 7. Pixel composition

Per visible dot:

```rust
let bg = background_pixel();   // 0..=3, 0 = transparent
let sp = sprite_pixel();       // 0..=3, 0 = transparent

let colour_addr = if sp != 0 && (bg == 0 || !sprite_behind_bg) {
    0x3F10 | (sprite_palette << 2) | sp
} else if bg != 0 {
    0x3F00 | (bg_palette << 2) | bg
} else {
    0x3F00                    // backdrop
};

let mut index = palette_ram[colour_addr & 0x1F];

if mask.contains(PPUMASK::GRAYSCALE) {
    index &= 0x30;
}

framebuffer[y][x] = index;
```

| Background | Sprite | Result |
|---|---|---|
| transparent | transparent | backdrop `$3F00` |
| opaque | transparent | background |
| transparent | opaque | sprite |
| opaque | opaque | sprite, unless its priority bit says behind |

If `PPUMASK` bit 3 (background) or bit 4 (sprites) is clear, that layer contributes
nothing. If both are clear, every pixel is the backdrop colour.

Left-edge masking (`PPUMASK` bit 1 for background, bit 2 for sprites) hides `x < 8`.

---

## 8. Output

The PPU produces a **palette index** per pixel, not RGB:

```
256 × 240 = 61,440 bytes, one index (0–63) per pixel
```

Convert to RGBA once per frame, at present time:

```rust
for (i, &index) in fb.iter().enumerate() {
    let [r, g, b] = SYSTEM_PALETTE[index as usize & 0x3F];
    out[i * 4..i * 4 + 4].copy_from_slice(&[r, g, b, 0xFF]);
}
```

Keeping indices means snapshots stay small, palette swaps are free, and the debugger
can inspect indices directly.

---

## 9. Implementation stages

Each stage is independently testable and has a known set of things it gets wrong.

### Stage 0 — PPU bus and palette

`PpuBus` with CHR, nametables, palette; 14-bit `read`/`write`; `$3F00–$3FFF` mirroring;
palette entry-0 alias.

**Test:** `blargg_ppu_tests/palette_ram.nes`.

### Stage 1 — static render

`render(&self) -> FrameBuffer` computing the background from `PPUCTRL`, `PPUMASK`,
nametables, attributes, pattern tables, palette. No shift registers, no scroll, no
sprites.

**Works:** static screens, all memory decode, backdrop, pattern-table selection.

**Broken:** scrolling, sprites, mid-frame register changes, left-edge masking.

**Test:** unit tests per decode step; `full_palette.nes`; `nestest.nes` manual screen.

### Stage 2 — timing skeleton

`scanline`/`dot` counters, vblank at `(241, 1)`, pre-render at 261, NMI line and latch.

**Works:** ROMs reach their vblank code and start drawing.

**Broken:** output is still frame-static.

**Test:** `ppu_vbl_nmi.nes`.

### Stage 3 — background pipeline

Shift registers, 8-dot fetch cycle, `fine_x`, loopy updates at 256/257/280–304,
nametable mirroring.

**Works:** scrolling, split-screen effects.

**Broken:** sprites.

**Test:** scroll tests in `blargg_ppu_tests`; `240p Test Suite` scroll screens.

### Stage 4 — sprites

Evaluation, secondary OAM, fetch, output, priority, flips, 8×16 mode.

**Works:** complete picture.

**Broken:** sprite-0 hit, overflow edge cases.

**Test:** `sprite_hit_tests`, `sprite_overflow_tests`.

### Stage 5 — quirks

Sprite-0 hit timing, overflow flag bug, odd-frame skip, left-edge clipping, `$2007`
palette read behaviour.

---

## 10. Bug checklist

When the picture is wrong, check in this order:

| Symptom | Likely cause |
|---|---|
| Whole screen backdrop colour | `PPUMASK` bit 3 clear, or empty nametable |
| Tiles correct, colours wrong | attribute shift `((ty & 2) << 1) \| (tx & 2)` |
| Tiles look like noise | pattern bitplane offset (`+8`) or bit order (`7 - x`) |
| First tile of each row wrong | shift-register reload timing |
| Everything shifted 1–7 px | `fine_x` ignored or applied at the wrong end |
| Wrong nametable | `PPUCTRL` bits 0–1, or mirroring not applied |
| Bottom rows garbage | pre-render line not fetching |
| Last line flickers | odd-frame skip |
| Sprite colours wrong | palette base `$3F10` vs `$3F00` |
| Sprites behind everything | priority bit inverted |
| Leftmost 8 px always drawn | `PPUMASK` bits 1/2 ignored |
| Sprite 0 hit never fires | wrong dot, or transparency handling |

---

## 11. What "rendering" is not

- It is not "draw the screen after the frame". It is a per-dot pipeline; the picture is
  a side effect of the fetch schedule.
- It is not region dependent except in frame length and vblank duration. Both NTSC and
  PAL produce 256×240.
- It does not need RGB until present time. Indices are the real output.
- It does not need sprites to be useful. Background alone proves the entire memory path.
