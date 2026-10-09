use crate::cpu::op::{build_variants, OpTable};

use super::{Cpu, InstrVariants};

use super::Mode::{
    Absolute, AbsoluteX, AbsoluteXWithCross, AbsoluteYWithCross, Immediate, Implied, IndirectX,
    IndirectYWithCross, ZeroPage, ZeroPageX,
};

pub const fn build(table: &mut OpTable) {
    build_variants(table, ADC, "ADC");
    build_variants(table, SBC, "SBC");
    build_variants(table, INC, "INC");
    build_variants(table, DEC, "DEC");
    build_variants(table, INX, "INX");
    build_variants(table, INY, "INY");
    build_variants(table, DEX, "DEX");
    build_variants(table, DEY, "DEY");
}

const ADC: InstrVariants<8> = [
    (Cpu::adc, Immediate, 0x69, 2),
    (Cpu::adc, ZeroPage, 0x65, 3),
    (Cpu::adc, ZeroPageX, 0x75, 4),
    (Cpu::adc, Absolute, 0x6D, 4),
    (Cpu::adc, AbsoluteXWithCross, 0x7D, 4),
    (Cpu::adc, AbsoluteYWithCross, 0x79, 4),
    (Cpu::adc, IndirectX, 0x61, 6),
    (Cpu::adc, IndirectYWithCross, 0x71, 5),
];

const SBC: InstrVariants<8> = [
    (Cpu::sbc, Immediate, 0xE9, 2),
    (Cpu::sbc, ZeroPage, 0xE5, 3),
    (Cpu::sbc, ZeroPageX, 0xF5, 4),
    (Cpu::sbc, Absolute, 0xED, 4),
    (Cpu::sbc, AbsoluteXWithCross, 0xFD, 4),
    (Cpu::sbc, AbsoluteYWithCross, 0xF9, 4),
    (Cpu::sbc, IndirectX, 0xE1, 6),
    (Cpu::sbc, IndirectYWithCross, 0xF1, 5),
];

const INC: InstrVariants<4> = [
    (Cpu::inc, ZeroPage, 0xE6, 5),
    (Cpu::inc, ZeroPageX, 0xF6, 6),
    (Cpu::inc, Absolute, 0xEE, 6),
    (Cpu::inc, AbsoluteX, 0xFE, 7),
];

const DEC: InstrVariants<4> = [
    (Cpu::dec, ZeroPage, 0xC6, 5),
    (Cpu::dec, ZeroPageX, 0xD6, 6),
    (Cpu::dec, Absolute, 0xCE, 6),
    (Cpu::dec, AbsoluteX, 0xDE, 7),
];

const INX: InstrVariants<1> = [(Cpu::inx, Implied, 0xE8, 2)];
const INY: InstrVariants<1> = [(Cpu::iny, Implied, 0xC8, 2)];
const DEX: InstrVariants<1> = [(Cpu::dex, Implied, 0xCA, 2)];
const DEY: InstrVariants<1> = [(Cpu::dey, Implied, 0x88, 2)];
