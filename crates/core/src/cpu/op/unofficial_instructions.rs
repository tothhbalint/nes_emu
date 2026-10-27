use super::Cpu;
use super::Mode::{
    Absolute, AbsoluteX, AbsoluteXWithCross, AbsoluteY, AbsoluteYWithCross, Immediate, IndirectX,
    IndirectY, ZeroPage, ZeroPageX, ZeroPageY,
};
use super::{build_variants, InstrVariants, OpTable};

pub const fn build(table: &mut OpTable) {
    build_variants(table, SLO, "SLO");
    build_variants(table, RLA, "RLA");
    build_variants(table, SRE, "SRE");
    build_variants(table, RRA, "RRA");
    build_variants(table, DCP, "DCP");
    build_variants(table, ISC, "ISC");
    build_variants(table, SAX, "SAX");
    build_variants(table, LAX, "LAX");
    build_variants(table, LXA, "LXA");
    build_variants(table, ALR, "ALR");
    build_variants(table, ANC, "ANC");
    build_variants(table, ARR, "ARR");
    build_variants(table, AXS, "AXS");
    build_variants(table, SBC_IMMEDIATE, "SBC");
    build_variants(table, SHX, "SHX");
    build_variants(table, SHY, "SHY");
    build_variants(table, NOP, "NOP");
}

// Read-modify-write instructions. These take a fixed cycle count, so the plain
// (non-`WithCross`) modes are used even with indexed addressing.

const SLO: InstrVariants<7> = [
    (Cpu::slo, IndirectX, 0x03, 8),
    (Cpu::slo, ZeroPage, 0x07, 5),
    (Cpu::slo, Absolute, 0x0F, 6),
    (Cpu::slo, IndirectY, 0x13, 8),
    (Cpu::slo, ZeroPageX, 0x17, 6),
    (Cpu::slo, AbsoluteY, 0x1B, 7),
    (Cpu::slo, AbsoluteX, 0x1F, 7),
];

const RLA: InstrVariants<7> = [
    (Cpu::rla, IndirectX, 0x23, 8),
    (Cpu::rla, ZeroPage, 0x27, 5),
    (Cpu::rla, Absolute, 0x2F, 6),
    (Cpu::rla, IndirectY, 0x33, 8),
    (Cpu::rla, ZeroPageX, 0x37, 6),
    (Cpu::rla, AbsoluteY, 0x3B, 7),
    (Cpu::rla, AbsoluteX, 0x3F, 7),
];

const SRE: InstrVariants<7> = [
    (Cpu::sre, IndirectX, 0x43, 8),
    (Cpu::sre, ZeroPage, 0x47, 5),
    (Cpu::sre, Absolute, 0x4F, 6),
    (Cpu::sre, IndirectY, 0x53, 8),
    (Cpu::sre, ZeroPageX, 0x57, 6),
    (Cpu::sre, AbsoluteY, 0x5B, 7),
    (Cpu::sre, AbsoluteX, 0x5F, 7),
];

const RRA: InstrVariants<7> = [
    (Cpu::rra, IndirectX, 0x63, 8),
    (Cpu::rra, ZeroPage, 0x67, 5),
    (Cpu::rra, Absolute, 0x6F, 6),
    (Cpu::rra, IndirectY, 0x73, 8),
    (Cpu::rra, ZeroPageX, 0x77, 6),
    (Cpu::rra, AbsoluteY, 0x7B, 7),
    (Cpu::rra, AbsoluteX, 0x7F, 7),
];

const DCP: InstrVariants<7> = [
    (Cpu::dcp, IndirectX, 0xC3, 8),
    (Cpu::dcp, ZeroPage, 0xC7, 5),
    (Cpu::dcp, Absolute, 0xCF, 6),
    (Cpu::dcp, IndirectY, 0xD3, 8),
    (Cpu::dcp, ZeroPageX, 0xD7, 6),
    (Cpu::dcp, AbsoluteY, 0xDB, 7),
    (Cpu::dcp, AbsoluteX, 0xDF, 7),
];

const ISC: InstrVariants<7> = [
    (Cpu::isc, IndirectX, 0xE3, 8),
    (Cpu::isc, ZeroPage, 0xE7, 5),
    (Cpu::isc, Absolute, 0xEF, 6),
    (Cpu::isc, IndirectY, 0xF3, 8),
    (Cpu::isc, ZeroPageX, 0xF7, 6),
    (Cpu::isc, AbsoluteY, 0xFB, 7),
    (Cpu::isc, AbsoluteX, 0xFF, 7),
];

// Stores

const SAX: InstrVariants<4> = [
    (Cpu::sax, IndirectX, 0x83, 6),
    (Cpu::sax, ZeroPage, 0x87, 3),
    (Cpu::sax, Absolute, 0x8F, 4),
    (Cpu::sax, ZeroPageY, 0x97, 4),
];

// Loads

const LAX: InstrVariants<6> = [
    (Cpu::lax, IndirectX, 0xA3, 6),
    (Cpu::lax, ZeroPage, 0xA7, 3),
    (Cpu::lax, Absolute, 0xAF, 4),
    (Cpu::lax, IndirectY, 0xB3, 6),
    (Cpu::lax, ZeroPageY, 0xB7, 4),
    (Cpu::lax, AbsoluteYWithCross, 0xBF, 4),
];

// Immediate-only instructions

const ALR: InstrVariants<1> = [(Cpu::alr, Immediate, 0x4B, 2)];

const ANC: InstrVariants<2> = [
    (Cpu::anc, Immediate, 0x0B, 2),
    (Cpu::anc, Immediate, 0x2B, 2),
];

const ARR: InstrVariants<1> = [(Cpu::arr, Immediate, 0x6B, 2)];

const AXS: InstrVariants<1> = [(Cpu::axs, Immediate, 0xCB, 2)];

const LXA: InstrVariants<1> = [(Cpu::lxa, Immediate, 0xAB, 2)];

// $EB is a duplicate encoding of SBC #i.
const SBC_IMMEDIATE: InstrVariants<1> = [(Cpu::sbc, Immediate, 0xEB, 2)];

// Mis implemented addressings
//
const SHX: InstrVariants<1> = [(Cpu::shx, AbsoluteY, 0x9E, 5)];

const SHY: InstrVariants<1> = [(Cpu::shy, AbsoluteX, 0x9C, 5)];

// Illegal NOPs. They differ only in how many operand bytes they consume.

const NOP: InstrVariants<21> = [
    (Cpu::nop, Immediate, 0x80, 2),
    (Cpu::nop, Immediate, 0x82, 2),
    (Cpu::nop, Immediate, 0x89, 2),
    (Cpu::nop, Immediate, 0xC2, 2),
    (Cpu::nop, Immediate, 0xE2, 2),
    (Cpu::nop, ZeroPage, 0x04, 3),
    (Cpu::nop, ZeroPage, 0x44, 3),
    (Cpu::nop, ZeroPage, 0x64, 3),
    (Cpu::nop, ZeroPageX, 0x14, 4),
    (Cpu::nop, ZeroPageX, 0x34, 4),
    (Cpu::nop, ZeroPageX, 0x54, 4),
    (Cpu::nop, ZeroPageX, 0x74, 4),
    (Cpu::nop, ZeroPageX, 0xD4, 4),
    (Cpu::nop, ZeroPageX, 0xF4, 4),
    (Cpu::nop, Absolute, 0x0C, 4),
    (Cpu::nop, AbsoluteXWithCross, 0x1C, 4),
    (Cpu::nop, AbsoluteXWithCross, 0x3C, 4),
    (Cpu::nop, AbsoluteXWithCross, 0x5C, 4),
    (Cpu::nop, AbsoluteXWithCross, 0x7C, 4),
    (Cpu::nop, AbsoluteXWithCross, 0xDC, 4),
    (Cpu::nop, AbsoluteXWithCross, 0xFC, 4),
];
