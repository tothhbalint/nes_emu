use super::Cpu;
use super::Mode::Implied;
use super::{build_variants, InstrVariants, OpTable};

pub const fn build(table: &mut OpTable) {
    build_variants(table, PHA, "PHA");
    build_variants(table, PLA, "PLA");
    build_variants(table, PHP, "PHP");
    build_variants(table, PLP, "PLP");
    build_variants(table, TXS, "TXS");
    build_variants(table, TSX, "TSX");
}

const PHA: InstrVariants<1> = [(Cpu::pha, Implied, 0x48, 3)];

const PLA: InstrVariants<1> = [(Cpu::pla, Implied, 0x68, 4)];

const PHP: InstrVariants<1> = [(Cpu::php, Implied, 0x08, 3)];

const PLP: InstrVariants<1> = [(Cpu::plp, Implied, 0x28, 4)];

const TXS: InstrVariants<1> = [(Cpu::txs, Implied, 0x9A, 2)];

const TSX: InstrVariants<1> = [(Cpu::tsx, Implied, 0xBA, 2)];
