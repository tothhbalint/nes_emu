use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

const OAM_SIZE: usize = 256;

// A: flat bytes
struct OamFlat {
    data: [u8; OAM_SIZE],
}

impl OamFlat {
    fn new() -> Self {
        OamFlat {
            data: [0; OAM_SIZE],
        }
    }
    #[inline]
    fn write_at(&mut self, addr: u8, val: u8) {
        self.data[addr as usize] = val;
    }
    #[inline]
    fn sprite_y(&self, i: usize) -> u8 {
        self.data[(i & 63) * 4]
    }
}

// B: 2D matrix
struct Oam2d {
    data: [[u8; 4]; 64],
}

impl Oam2d {
    fn new() -> Self {
        Oam2d { data: [[0; 4]; 64] }
    }
    #[inline]
    fn write_at(&mut self, addr: u8, val: u8) {
        self.data[(addr >> 2) as usize][(addr & 3) as usize] = val;
    }
    #[inline]
    fn sprite_y(&self, i: usize) -> u8 {
        self.data[i & 63][0]
    }
}

// C: array-of-arrays wrapped in a struct
#[derive(Clone, Copy, Default)]
struct SpriteArr {
    bytes: [u8; 4],
}

struct OamStruct {
    data: [SpriteArr; 64],
}

impl OamStruct {
    fn new() -> Self {
        OamStruct {
            data: [SpriteArr::default(); 64],
        }
    }
    #[inline]
    fn write_at(&mut self, addr: u8, val: u8) {
        self.data[(addr >> 2) as usize].bytes[(addr & 3) as usize] = val;
    }
    #[inline]
    fn sprite_y(&self, i: usize) -> u8 {
        self.data[i & 63].bytes[0]
    }
}

// D: named fields
#[derive(Clone, Copy, Default)]
struct SpriteNamed {
    y: u8,
    tile: u8,
    attr: u8,
    x: u8,
}

struct OamNamed {
    data: [SpriteNamed; 64],
}

impl OamNamed {
    fn new() -> Self {
        OamNamed {
            data: [SpriteNamed::default(); 64],
        }
    }
    #[inline]
    fn write_at(&mut self, addr: u8, val: u8) {
        let s = &mut self.data[(addr >> 2) as usize];
        match addr & 3 {
            0 => s.y = val,
            1 => s.tile = val,
            2 => s.attr = val,
            _ => s.x = val,
        }
    }
    #[inline]
    fn sprite_y(&self, i: usize) -> u8 {
        self.data[i & 63].y
    }
}

fn oam_dma(c: &mut Criterion) {
    let page = [0xA5u8; 256];
    let mut g = c.benchmark_group("oam_write_256");

    g.bench_function("flat", |b| {
        let mut oam = OamFlat::new();
        b.iter(|| {
            for (i, &byte) in black_box(&page).iter().enumerate() {
                oam.write_at(i as u8, byte);
            }
            black_box(&oam.data); // all 256 bytes must be materialised
        })
    });

    g.bench_function("2d", |b| {
        let mut oam = Oam2d::new();
        b.iter(|| {
            for (i, &byte) in black_box(&page).iter().enumerate() {
                oam.write_at(i as u8, byte);
            }
            black_box(oam.data[0][0])
        })
    });

    g.bench_function("struct_arr", |b| {
        let mut oam = OamStruct::new();
        b.iter(|| {
            for (i, &byte) in black_box(&page).iter().enumerate() {
                oam.write_at(i as u8, byte);
            }
            black_box(oam.data[0].bytes[0])
        })
    });

    g.bench_function("struct_named", |b| {
        let mut oam = OamNamed::new();
        b.iter(|| {
            for (i, &byte) in black_box(&page).iter().enumerate() {
                oam.write_at(i as u8, byte);
            }
            black_box(oam.data[0].y)
        })
    });

    g.finish();
}

fn sprite_scan(c: &mut Criterion) {
    let mut g = c.benchmark_group("sprite_scan_64");

    let flat = OamFlat::new();
    g.bench_function("flat", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for i in 0..64 {
                acc += flat.sprite_y(black_box(i)) as u32;
            }
            black_box(acc)
        })
    });

    let oam2d = Oam2d::new();
    g.bench_function("2d", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for i in 0..64 {
                acc += oam2d.sprite_y(black_box(i)) as u32;
            }
            black_box(acc)
        })
    });

    let s = OamStruct::new();
    g.bench_function("struct_arr", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for i in 0..64 {
                acc += s.sprite_y(black_box(i)) as u32;
            }
            black_box(acc)
        })
    });

    let n = OamNamed::new();
    g.bench_function("struct_named", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for i in 0..64 {
                acc += n.sprite_y(black_box(i)) as u32;
            }
            black_box(acc)
        })
    });

    g.finish();
}

criterion_group!(benches, oam_dma, sprite_scan);
criterion_main!(benches);
