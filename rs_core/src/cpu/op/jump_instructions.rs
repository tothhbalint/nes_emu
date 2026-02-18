use super::Cpu;
use super::Mode::{Absolute, Implied, Indirect};
use super::{build_variants, InstrVariants, OpTable};

pub const fn build(table: &mut OpTable) {
    build_variants(table, JMP, "JMP");
    build_variants(table, JSR, "JSR");
    build_variants(table, RTS, "RTS");
    build_variants(table, BRK, "BRK");
    build_variants(table, RTI, "RTI");
}

const JMP: InstrVariants<2> = [(Cpu::jmp, Absolute, 0x4C, 3), (Cpu::jmp, Indirect, 0x6C, 5)];

const JSR: InstrVariants<1> = [(Cpu::jsr, Absolute, 0x20, 6)];

const RTS: InstrVariants<1> = [(Cpu::rts, Implied, 0x60, 6)];

const BRK: InstrVariants<1> = [(Cpu::brk, Implied, 0x00, 7)];

const RTI: InstrVariants<1> = [(Cpu::rti, Implied, 0x40, 6)];
