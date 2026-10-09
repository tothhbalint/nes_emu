use criterion::{black_box, criterion_group, criterion_main, Criterion};

struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    p: u8,
}

impl Cpu {
    fn new() -> Self {
        Cpu { a: 0, x: 0, y: 0, pc: 0x8000, sp: 0xFD, p: 0x24 }
    }
}

// Realistic instruction implementations
#[inline(never)]
fn adc(cpu: &mut Cpu, value: u8) {
    let carry = if cpu.p & 0x01 != 0 { 1 } else { 0 };
    let sum = cpu.a as u16 + value as u16 + carry;
    cpu.p = (cpu.p & !0x01) | if sum > 0xFF { 0x01 } else { 0x00 };
    let result = sum as u8;
    let overflow = (cpu.a ^ result) & (value ^ result) & 0x80 != 0;
    cpu.p = (cpu.p & !0x40) | if overflow { 0x40 } else { 0x00 };
    cpu.a = result;
    cpu.p = (cpu.p & !0x02) | if result == 0 { 0x02 } else { 0x00 };
    cpu.p = (cpu.p & !0x80) | (result & 0x80);
}

#[inline(never)]
fn and(cpu: &mut Cpu, value: u8) {
    cpu.a &= value;
    cpu.p = (cpu.p & !0x02) | if cpu.a == 0 { 0x02 } else { 0x00 };
    cpu.p = (cpu.p & !0x80) | (cpu.a & 0x80);
}

#[inline(never)]
fn eor(cpu: &mut Cpu, value: u8) {
    cpu.a ^= value;
    cpu.p = (cpu.p & !0x02) | if cpu.a == 0 { 0x02 } else { 0x00 };
    cpu.p = (cpu.p & !0x80) | (cpu.a & 0x80);
}

#[inline(never)]
fn ora(cpu: &mut Cpu, value: u8) {
    cpu.a |= value;
    cpu.p = (cpu.p & !0x02) | if cpu.a == 0 { 0x02 } else { 0x00 };
    cpu.p = (cpu.p & !0x80) | (cpu.a & 0x80);
}

#[inline(never)]
fn lda(cpu: &mut Cpu, value: u8) {
    cpu.a = value;
    cpu.p = (cpu.p & !0x02) | if value == 0 { 0x02 } else { 0x00 };
    cpu.p = (cpu.p & !0x80) | (value & 0x80);
}

#[inline(never)]
fn ldx(cpu: &mut Cpu, value: u8) {
    cpu.x = value;
    cpu.p = (cpu.p & !0x02) | if value == 0 { 0x02 } else { 0x00 };
    cpu.p = (cpu.p & !0x80) | (value & 0x80);
}

#[inline(never)]
fn ldy(cpu: &mut Cpu, value: u8) {
    cpu.y = value;
    cpu.p = (cpu.p & !0x02) | if value == 0 { 0x02 } else { 0x00 };
    cpu.p = (cpu.p & !0x80) | (value & 0x80);
}

#[inline(never)]
fn cmp(cpu: &mut Cpu, value: u8) {
    let result = cpu.a.wrapping_sub(value);
    cpu.p = (cpu.p & !0x01) | if cpu.a >= value { 0x01 } else { 0x00 };
    cpu.p = (cpu.p & !0x02) | if result == 0 { 0x02 } else { 0x00 };
    cpu.p = (cpu.p & !0x80) | (result & 0x80);
}

#[inline(never)]
fn sbc(cpu: &mut Cpu, value: u8) {
    let carry = if cpu.p & 0x01 != 0 { 0 } else { 1 };
    let diff = cpu.a as i16 - value as i16 - carry as i16;
    cpu.p = (cpu.p & !0x01) | if diff >= 0 { 0x01 } else { 0x00 };
    let result = diff as u8;
    let overflow = ((cpu.a ^ result) & ((255 - value) ^ result) & 0x80) != 0;
    cpu.p = (cpu.p & !0x40) | if overflow { 0x40 } else { 0x00 };
    cpu.a = result;
    cpu.p = (cpu.p & !0x02) | if result == 0 { 0x02 } else { 0x00 };
    cpu.p = (cpu.p & !0x80) | (result & 0x80);
}

#[inline(never)]
fn nop(_cpu: &mut Cpu, _value: u8) {}

// ============================================================================
// MATCH APPROACH - ALL 256 OPCODES
// ============================================================================

#[inline(never)]
fn step_match(cpu: &mut Cpu, opcode: u8, value: u8) {
    match opcode {
        // ORA
        0x01 => ora(cpu, value), 0x05 => ora(cpu, value), 0x09 => ora(cpu, value),
        0x0D => ora(cpu, value), 0x11 => ora(cpu, value), 0x15 => ora(cpu, value),
        0x19 => ora(cpu, value), 0x1D => ora(cpu, value),
        
        // AND
        0x21 => and(cpu, value), 0x25 => and(cpu, value), 0x29 => and(cpu, value),
        0x2D => and(cpu, value), 0x31 => and(cpu, value), 0x35 => and(cpu, value),
        0x39 => and(cpu, value), 0x3D => and(cpu, value),
        
        // EOR
        0x41 => eor(cpu, value), 0x45 => eor(cpu, value), 0x49 => eor(cpu, value),
        0x4D => eor(cpu, value), 0x51 => eor(cpu, value), 0x55 => eor(cpu, value),
        0x59 => eor(cpu, value), 0x5D => eor(cpu, value),
        
        // ADC
        0x61 => adc(cpu, value), 0x65 => adc(cpu, value), 0x69 => adc(cpu, value),
        0x6D => adc(cpu, value), 0x71 => adc(cpu, value), 0x75 => adc(cpu, value),
        0x79 => adc(cpu, value), 0x7D => adc(cpu, value),
        
        // LDA
        0xA1 => lda(cpu, value), 0xA5 => lda(cpu, value), 0xA9 => lda(cpu, value),
        0xAD => lda(cpu, value), 0xB1 => lda(cpu, value), 0xB5 => lda(cpu, value),
        0xB9 => lda(cpu, value), 0xBD => lda(cpu, value),
        
        // CMP
        0xC1 => cmp(cpu, value), 0xC5 => cmp(cpu, value), 0xC9 => cmp(cpu, value),
        0xCD => cmp(cpu, value), 0xD1 => cmp(cpu, value), 0xD5 => cmp(cpu, value),
        0xD9 => cmp(cpu, value), 0xDD => cmp(cpu, value),
        
        // SBC
        0xE1 => sbc(cpu, value), 0xE5 => sbc(cpu, value), 0xE9 => sbc(cpu, value),
        0xED => sbc(cpu, value), 0xF1 => sbc(cpu, value), 0xF5 => sbc(cpu, value),
        0xF9 => sbc(cpu, value), 0xFD => sbc(cpu, value),
        
        // LDX
        0xA2 => ldx(cpu, value), 0xA6 => ldx(cpu, value), 0xAE => ldx(cpu, value),
        0xB6 => ldx(cpu, value), 0xBE => ldx(cpu, value),
        
        // LDY
        0xA0 => ldy(cpu, value), 0xA4 => ldy(cpu, value), 0xAC => ldy(cpu, value),
        0xB4 => ldy(cpu, value), 0xBC => ldy(cpu, value),
        
        // Everything else - NOP for now (fill in rest of 256 opcodes)
        _ => nop(cpu, value),
    }
}

// ============================================================================
// TABLE APPROACH - FULL 256 ENTRIES
// ============================================================================

type Handler = fn(&mut Cpu, u8);

fn make_table() -> [Handler; 256] {
    let mut table = [nop as Handler; 256];
    
    // ORA
    table[0x01] = ora; table[0x05] = ora; table[0x09] = ora;
    table[0x0D] = ora; table[0x11] = ora; table[0x15] = ora;
    table[0x19] = ora; table[0x1D] = ora;
    
    // AND
    table[0x21] = and; table[0x25] = and; table[0x29] = and;
    table[0x2D] = and; table[0x31] = and; table[0x35] = and;
    table[0x39] = and; table[0x3D] = and;
    
    // EOR
    table[0x41] = eor; table[0x45] = eor; table[0x49] = eor;
    table[0x4D] = eor; table[0x51] = eor; table[0x55] = eor;
    table[0x59] = eor; table[0x5D] = eor;
    
    // ADC
    table[0x61] = adc; table[0x65] = adc; table[0x69] = adc;
    table[0x6D] = adc; table[0x71] = adc; table[0x75] = adc;
    table[0x79] = adc; table[0x7D] = adc;
    
    // LDA
    table[0xA1] = lda; table[0xA5] = lda; table[0xA9] = lda;
    table[0xAD] = lda; table[0xB1] = lda; table[0xB5] = lda;
    table[0xB9] = lda; table[0xBD] = lda;
    
    // CMP
    table[0xC1] = cmp; table[0xC5] = cmp; table[0xC9] = cmp;
    table[0xCD] = cmp; table[0xD1] = cmp; table[0xD5] = cmp;
    table[0xD9] = cmp; table[0xDD] = cmp;
    
    // SBC
    table[0xE1] = sbc; table[0xE5] = sbc; table[0xE9] = sbc;
    table[0xED] = sbc; table[0xF1] = sbc; table[0xF5] = sbc;
    table[0xF9] = sbc; table[0xFD] = sbc;
    
    // LDX
    table[0xA2] = ldx; table[0xA6] = ldx; table[0xAE] = ldx;
    table[0xB6] = ldx; table[0xBE] = ldx;
    
    // LDY
    table[0xA0] = ldy; table[0xA4] = ldy; table[0xAC] = ldy;
    table[0xB4] = ldy; table[0xBC] = ldy;
    
    table
}

#[inline(never)]
fn step_table(cpu: &mut Cpu, opcode: u8, value: u8, table: &[Handler; 256]) {
    table[opcode as usize](cpu, value);
}

// ============================================================================
// BENCHMARKS
// ============================================================================

fn bench_match(c: &mut Criterion) {
    c.bench_function("match_full_256", |b| {
        let mut cpu = Cpu::new();
        b.iter(|| {
            for opcode in 0..=255u8 {
                step_match(black_box(&mut cpu), black_box(opcode), black_box(42));
            }
        });
    });
}

fn bench_table(c: &mut Criterion) {
    let table = make_table();
    c.bench_function("table_full_256", |b| {
        let mut cpu = Cpu::new();
        b.iter(|| {
            for opcode in 0..=255u8 {
                step_table(black_box(&mut cpu), black_box(opcode), black_box(42), &table);
            }
        });
    });
}

criterion_group!(benches, bench_match, bench_table);
criterion_main!(benches);
