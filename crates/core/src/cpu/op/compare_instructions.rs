use super::Cpu;
use super::Mode::{
    Absolute, AbsoluteXWithCross, AbsoluteYWithCross, Immediate, IndirectX, IndirectYWithCross,
    ZeroPage, ZeroPageX,
};
use super::{build_variants, InstrVariants, OpTable};

pub const fn build(table: &mut OpTable) {
    build_variants(table, CMP, "CMP");
    build_variants(table, CPX, "CPX");
    build_variants(table, CPY, "CPY");
}

const CMP: InstrVariants<8> = [
    (Cpu::cmp, Immediate, 0xC9, 2),
    (Cpu::cmp, ZeroPage, 0xC5, 3),
    (Cpu::cmp, ZeroPageX, 0xD5, 4),
    (Cpu::cmp, Absolute, 0xCD, 4),
    (Cpu::cmp, AbsoluteXWithCross, 0xDD, 4),
    (Cpu::cmp, AbsoluteYWithCross, 0xD9, 4),
    (Cpu::cmp, IndirectX, 0xC1, 6),
    (Cpu::cmp, IndirectYWithCross, 0xD1, 5),
];
const CPX: InstrVariants<3> = [
    (Cpu::cpx, Immediate, 0xE0, 2),
    (Cpu::cpx, ZeroPage, 0xE4, 3),
    (Cpu::cpx, Absolute, 0xEC, 4),
];
const CPY: InstrVariants<3> = [
    (Cpu::cpy, Immediate, 0xC0, 2),
    (Cpu::cpy, ZeroPage, 0xC4, 3),
    (Cpu::cpy, Absolute, 0xCC, 4),
];
