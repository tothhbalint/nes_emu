use super::Bus;
use super::Cpu;

type InstrFn = fn(&mut Cpu, &mut Bus, u16);
type Instr = (InstrFn, Mode, usize, u8);
type InstrVariants<const N: usize> = [Instr; N];
type OpTable = [Op; 256];

pub(super) const fn build_variants<const N: usize>(
    table: &mut OpTable,
    variants: InstrVariants<N>,
    name: &'static str,
) {
    let mut i = 0;
    while i < N {
        let (exec, addr_mode, opcode, base_cycles) = variants[i];
        table[opcode] = Op {
            exec,
            addr_mode,
            base_cycles,
            #[cfg(debug_assertions)]
            name,
        };
        i += 1;
    }
}

pub mod addressing;
use addressing::Mode;

#[derive(Copy, Clone)]
pub(super) struct Op {
    pub(super) exec: InstrFn,
    pub(super) addr_mode: Mode,
    pub(super) base_cycles: u8,
    #[cfg(debug_assertions)]
    pub name: &'static str,
}

mod access_instructions;
mod arithmetic_instructions;
mod bitwise_instructions;
mod branch_instructions;
mod compare_instructions;
mod flags_instructions;
mod jump_instructions;
mod shift_instructions;
mod stack_instructions;
mod transfer_instructions;

// Build lookup table
pub(super) const fn make_op_table() -> [Op; 256] {
    let mut table = [Op {
        exec: Cpu::unoff,
        addr_mode: Mode::Implied,
        base_cycles: 0,
        #[cfg(debug_assertions)]
        name: "U",
    }; 256];

    access_instructions::build(&mut table);
    transfer_instructions::build(&mut table);
    arithmetic_instructions::build(&mut table);
    shift_instructions::build(&mut table);
    bitwise_instructions::build(&mut table);
    compare_instructions::build(&mut table);
    branch_instructions::build(&mut table);
    jump_instructions::build(&mut table);
    stack_instructions::build(&mut table);
    flags_instructions::build(&mut table);

    table[0xEA] = Op {
        exec: Cpu::nop,
        addr_mode: Mode::Implied,
        base_cycles: 2,
        #[cfg(debug_assertions)]
        name: "NOP",
    };

    table
}
