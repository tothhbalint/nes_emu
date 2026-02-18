use super::Cpu;
use super::Mode::Implied;
use super::{build_variants, InstrVariants, OpTable};

pub const fn build(table: &mut OpTable) {
    build_variants(table, CLC, "CLC");
    build_variants(table, SEC, "SEC");
    build_variants(table, CLI, "CLI");
    build_variants(table, SEI, "SEI");
    build_variants(table, CLD, "CLD");
    build_variants(table, SED, "SED");
    build_variants(table, CLV, "CLV");
}

const CLC: InstrVariants<1> = [(Cpu::clc, Implied, 0x18, 2)];

const SEC: InstrVariants<1> = [(Cpu::sec, Implied, 0x38, 2)];

const CLI: InstrVariants<1> = [(Cpu::cli, Implied, 0x58, 2)];

const SEI: InstrVariants<1> = [(Cpu::sei, Implied, 0x78, 2)];

const CLD: InstrVariants<1> = [(Cpu::cld, Implied, 0xD8, 2)];

const SED: InstrVariants<1> = [(Cpu::sed, Implied, 0xF8, 2)];

const CLV: InstrVariants<1> = [(Cpu::clv, Implied, 0xB8, 2)];
