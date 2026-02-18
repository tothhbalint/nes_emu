use super::Mode::{Absolute, AbsoluteX, Accumulator, ZeroPage, ZeroPageX};
use super::{build_variants, Cpu, Instr, InstrFn, OpTable};

pub const fn build(table: &mut OpTable) {
    build_variants(table, ASL, "ASL");
    build_variants(table, LSR, "LSR");
    build_variants(table, ROL, "ROL");
    build_variants(table, ROR, "ROR");
}

const ASL: [Instr; 5] = [
    (Cpu::asl_a, Accumulator, 0x0A, 2),
    (Cpu::asl, ZeroPage, 0x06, 5),
    (Cpu::asl, ZeroPageX, 0x16, 6),
    (Cpu::asl, Absolute, 0x0E, 6),
    (Cpu::asl, AbsoluteX, 0x1E, 7),
];

const LSR: [Instr; 5] = [
    (Cpu::lsr_a, Accumulator, 0x4A, 2),
    (Cpu::lsr, ZeroPage, 0x46, 5),
    (Cpu::lsr, ZeroPageX, 0x56, 6),
    (Cpu::lsr, Absolute, 0x4E, 6),
    (Cpu::lsr, AbsoluteX, 0x5E, 7),
];

const ROL: [Instr; 5] = [
    (Cpu::rol_a, Accumulator, 0x2A, 2),
    (Cpu::rol, ZeroPage, 0x26, 5),
    (Cpu::rol, ZeroPageX, 0x36, 6),
    (Cpu::rol, Absolute, 0x2E, 6),
    (Cpu::rol, AbsoluteX, 0x3E, 7),
];

const ROR: [Instr; 5] = [
    (Cpu::ror_a, Accumulator, 0x6A, 2),
    (Cpu::ror, ZeroPage, 0x66, 5),
    (Cpu::ror, ZeroPageX, 0x76, 6),
    (Cpu::ror, Absolute, 0x6E, 6),
    (Cpu::ror, AbsoluteX, 0x7E, 7),
];
