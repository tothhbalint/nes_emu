use super::Cpu;
use super::Mode::{
    Absolute, AbsoluteXWithCross, AbsoluteYWithCross, Immediate, IndirectX, IndirectYWithCross,
    ZeroPage, ZeroPageX,
};
use super::{build_variants, InstrVariants, OpTable};

pub const fn build(table: &mut OpTable) {
    build_variants(table, AND, "AND");
    build_variants(table, ORA, "ORA");
    build_variants(table, EOR, "EOR");
    build_variants(table, BIT, "BIT");
}

const AND: InstrVariants<8> = [
    (Cpu::and, Immediate, 0x29, 2),
    (Cpu::and, ZeroPage, 0x25, 3),
    (Cpu::and, ZeroPageX, 0x35, 4),
    (Cpu::and, Absolute, 0x2D, 4),
    (Cpu::and, AbsoluteXWithCross, 0x3D, 4),
    (Cpu::and, AbsoluteYWithCross, 0x39, 4),
    (Cpu::and, IndirectX, 0x21, 6),
    (Cpu::and, IndirectYWithCross, 0x31, 5),
];

const ORA: InstrVariants<8> = [
    (Cpu::ora, Immediate, 0x09, 2),
    (Cpu::ora, ZeroPage, 0x05, 3),
    (Cpu::ora, ZeroPageX, 0x15, 4),
    (Cpu::ora, Absolute, 0x0D, 4),
    (Cpu::ora, AbsoluteXWithCross, 0x1D, 4),
    (Cpu::ora, AbsoluteYWithCross, 0x19, 4),
    (Cpu::ora, IndirectX, 0x01, 6),
    (Cpu::ora, IndirectYWithCross, 0x11, 5),
];

const EOR: InstrVariants<8> = [
    (Cpu::eor, Immediate, 0x49, 2),
    (Cpu::eor, ZeroPage, 0x45, 3),
    (Cpu::eor, ZeroPageX, 0x55, 4),
    (Cpu::eor, Absolute, 0x4D, 4),
    (Cpu::eor, AbsoluteXWithCross, 0x5D, 4),
    (Cpu::eor, AbsoluteYWithCross, 0x59, 4),
    (Cpu::eor, IndirectX, 0x41, 6),
    (Cpu::eor, IndirectYWithCross, 0x51, 5),
];

const BIT: InstrVariants<2> = [(Cpu::bit, ZeroPage, 0x24, 3), (Cpu::bit, Absolute, 0x2C, 4)];
