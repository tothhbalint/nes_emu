use crate::cpu::{
    op::{build_variants, InstrVariants, OpTable},
    Cpu,
};

use super::Mode::Implied;

pub const fn build(table: &mut OpTable) {
    build_variants(table, TAX, "TAX");
    build_variants(table, TXA, "TXA");
    build_variants(table, TYA, "TYA");
    build_variants(table, TAY, "TAY");
}

const TAX: InstrVariants<1> = [(Cpu::tax, Implied, 0xAA, 2)];
const TXA: InstrVariants<1> = [(Cpu::txa, Implied, 0x8A, 2)];
const TAY: InstrVariants<1> = [(Cpu::tay, Implied, 0xA8, 2)];
const TYA: InstrVariants<1> = [(Cpu::tya, Implied, 0x98, 2)];
