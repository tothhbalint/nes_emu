use crate::cpu::op::{build_variants, OpTable};

use super::Cpu;
use super::Mode::{
    Absolute, AbsoluteX, AbsoluteXWithCross, AbsoluteY, AbsoluteYWithCross, Immediate, IndirectX,
    IndirectY, IndirectYWithCross, ZeroPage, ZeroPageX, ZeroPageY,
};

use super::InstrVariants;

pub const fn build(table: &mut OpTable) {
    build_variants(table, LDA, "LDA");
    build_variants(table, STA, "STA");
    build_variants(table, LDX, "LDX");
    build_variants(table, LDY, "LDY");
    build_variants(table, STX, "STX");
    build_variants(table, STY, "STY");
}

const LDA: InstrVariants<8> = [
    (Cpu::lda, Immediate, 0xA9, 2),
    (Cpu::lda, ZeroPage, 0xA5, 3),
    (Cpu::lda, ZeroPageX, 0xB5, 4),
    (Cpu::lda, Absolute, 0xAD, 4),
    (Cpu::lda, AbsoluteXWithCross, 0xBD, 4),
    (Cpu::lda, AbsoluteYWithCross, 0xB9, 4),
    (Cpu::lda, IndirectX, 0xA1, 6),
    (Cpu::lda, IndirectYWithCross, 0xB1, 5),
];

const STA: InstrVariants<7> = [
    (Cpu::sta, ZeroPage, 0x85, 3),
    (Cpu::sta, ZeroPageX, 0x95, 4),
    (Cpu::sta, Absolute, 0x8D, 4),
    (Cpu::sta, AbsoluteX, 0x9D, 5),
    (Cpu::sta, AbsoluteY, 0x99, 5),
    (Cpu::sta, IndirectX, 0x81, 6),
    (Cpu::sta, IndirectY, 0x91, 6),
];

const LDX: InstrVariants<5> = [
    (Cpu::ldx, Immediate, 0xA2, 2),
    (Cpu::ldx, ZeroPage, 0xA6, 3),
    (Cpu::ldx, ZeroPageY, 0xB6, 4),
    (Cpu::ldx, Absolute, 0xAE, 4),
    (Cpu::ldx, AbsoluteYWithCross, 0xBE, 4),
];

const LDY: InstrVariants<5> = [
    (Cpu::ldy, Immediate, 0xA0, 2),
    (Cpu::ldy, ZeroPage, 0xA4, 3),
    (Cpu::ldy, ZeroPageX, 0xB4, 4),
    (Cpu::ldy, Absolute, 0xAC, 4),
    (Cpu::ldy, AbsoluteXWithCross, 0xBC, 4),
];

const STX: InstrVariants<3> = [
    (Cpu::stx, ZeroPage, 0x86, 3),
    (Cpu::stx, ZeroPageY, 0x96, 4),
    (Cpu::stx, Absolute, 0x8E, 4),
];

const STY: InstrVariants<3> = [
    (Cpu::sty, ZeroPage, 0x84, 3),
    (Cpu::sty, ZeroPageX, 0x94, 4),
    (Cpu::sty, Absolute, 0x8C, 4),
];
