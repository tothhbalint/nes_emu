use super::Mode::Relative;
use crate::cpu::op::{build_variants, Cpu, InstrVariants, OpTable};

pub const fn build(table: &mut OpTable) {
    build_variants(table, BCC, "BCC");
    build_variants(table, BCS, "BCS");
    build_variants(table, BEQ, "BEQ");
    build_variants(table, BNE, "BNE");
    build_variants(table, BPL, "BPL");
    build_variants(table, BMI, "BMI");
    build_variants(table, BVC, "BVC");
    build_variants(table, BVS, "BVS");
}

const BCC: InstrVariants<1> = [(Cpu::bcc, Relative, 0x90, 2)];
const BCS: InstrVariants<1> = [(Cpu::bcs, Relative, 0xB0, 2)];
const BEQ: InstrVariants<1> = [(Cpu::beq, Relative, 0xF0, 2)];
const BNE: InstrVariants<1> = [(Cpu::bne, Relative, 0xD0, 2)];
const BPL: InstrVariants<1> = [(Cpu::bpl, Relative, 0x10, 2)];
const BMI: InstrVariants<1> = [(Cpu::bmi, Relative, 0x30, 2)];
const BVC: InstrVariants<1> = [(Cpu::bvc, Relative, 0x50, 2)];
const BVS: InstrVariants<1> = [(Cpu::bvs, Relative, 0x70, 2)];
