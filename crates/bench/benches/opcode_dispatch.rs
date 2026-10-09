
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ============================================================================
// MATCH-BASED APPROACH
// ============================================================================

#[inline(never)]
fn adc_immediate() -> u8 {
    2
}
#[inline(never)]
fn lda_immediate() -> u8 {
    2
}
#[inline(never)]
fn sta_zero_page() -> u8 {
    3
}
#[inline(never)]
fn nop() -> u8 {
    2
}
#[inline(never)]
fn jmp_absolute() -> u8 {
    3
}

pub fn step_match(opcode: u8) -> u8 {
    match opcode {
        0x00 => nop(),
        0x01 => nop(),
        0x02 => nop(),
        0x03 => nop(),
        0x04 => nop(),
        0x05 => nop(),
        0x06 => nop(),
        0x07 => nop(),
        0x08 => nop(),
        0x09 => nop(),
        0x0A => nop(),
        0x0B => nop(),
        0x0C => nop(),
        0x0D => nop(),
        0x0E => nop(),
        0x0F => nop(),

        0x69 => adc_immediate(),
        0x6A => nop(),
        0x6B => nop(),
        0x6C => nop(),
        0x6D => adc_immediate(),
        0x6E => nop(),
        0x6F => nop(),

        0x85 => sta_zero_page(),
        0x86 => sta_zero_page(),
        0x87 => nop(),
        0x88 => nop(),
        0x89 => nop(),
        0x8A => nop(),
        0x8B => nop(),
        0x8C => sta_zero_page(),
        0x8D => sta_zero_page(),
        0x8E => nop(),
        0x8F => nop(),

        0xA9 => lda_immediate(),
        0xAA => nop(),
        0xAB => nop(),
        0xAC => lda_immediate(),
        0xAD => lda_immediate(),
        0xAE => nop(),
        0xAF => nop(),

        0x4C => jmp_absolute(),
        0x4D => nop(),
        0x4E => nop(),
        0x4F => nop(),

        // Fill in the rest with nops
        _ => nop(),
    }
}

// ============================================================================
// FUNCTION POINTER APPROACH
// ============================================================================

type Handler = fn() -> u8;

const OPCODE_TABLE: [Handler; 256] = [
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x00-0x07
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x08-0x0F
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x10-0x17
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x18-0x1F
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x20-0x27
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x28-0x2F
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x30-0x37
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x38-0x3F
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x40-0x47
    nop,
    nop,
    nop,
    nop,
    jmp_absolute,
    nop,
    nop,
    nop, // 0x48-0x4F (0x4C = JMP)
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x50-0x57
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x58-0x5F
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x60-0x67
    nop,
    adc_immediate,
    nop,
    nop,
    nop,
    adc_immediate,
    nop,
    nop, // 0x68-0x6F (0x69, 0x6D = ADC)
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x70-0x77
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x78-0x7F
    nop,
    nop,
    nop,
    nop,
    nop,
    sta_zero_page,
    sta_zero_page,
    nop, // 0x80-0x87
    nop,
    nop,
    nop,
    nop,
    sta_zero_page,
    sta_zero_page,
    nop,
    nop, // 0x88-0x8F
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x90-0x97
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0x98-0x9F
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xA0-0xA7
    nop,
    lda_immediate,
    nop,
    nop,
    lda_immediate,
    lda_immediate,
    nop,
    nop, // 0xA8-0xAF
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xB0-0xB7
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xB8-0xBF
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xC0-0xC7
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xC8-0xCF
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xD0-0xD7
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xD8-0xDF
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xE0-0xE7
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xE8-0xEF
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xF0-0xF7
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop, // 0xF8-0xFF
];

pub fn step_table(opcode: u8) -> u8 {
    OPCODE_TABLE[opcode as usize]()
}

// ============================================================================
// BENCHMARKS
// ============================================================================

fn bench_match(c: &mut Criterion) {
    c.bench_function("match_dispatch", |b| {
        b.iter(|| {
            let mut total = 0u64;
            for opcode in 0..=255u8 {
                total += step_match(black_box(opcode)) as u64;
            }
            black_box(total);
        });
    });
}

fn bench_table(c: &mut Criterion) {
    c.bench_function("table_dispatch", |b| {
        b.iter(|| {
            let mut total = 0u64;
            for opcode in 0..=255u8 {
                total += step_table(black_box(opcode)) as u64;
            }
            black_box(total);
        });
    });
}

criterion_group!(benches, bench_match, bench_table);
criterion_main!(benches);
