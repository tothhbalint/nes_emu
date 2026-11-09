/*
 * This file contains helper functions for decoding operations at given addresses
 */

#[cfg(feature = "debug")]
use super::DisasmEntry;

use super::Bus;
use super::{op::addressing::Mode, Op, OP_LOOKUP};

pub(super) fn decode_op(opcode: usize) -> Op {
    OP_LOOKUP[opcode]
}

#[cfg(feature = "debug")]
pub(super) fn disass_at_address(bus: &mut Bus, addr: u16) -> DisasmEntry {
    let opcode = bus.read(addr);

    let op = decode_op(opcode as usize);

    let len = mode_len(op.addr_mode);

    DisasmEntry {
        pc: (addr),
        opcode: [
            opcode,
            bus.read(addr.wrapping_add(1)),
            bus.read(addr.wrapping_add(2)),
        ],
        len,
        mnemonic: op.name,
    }
}

const fn mode_len(mode: Mode) -> u8 {
    use Mode::*;
    match mode {
        // 1 byte, no operand
        Implied | Accumulator => 1,

        // 2 bytes: opcode + 1 operand byte
        Immediate | ZeroPage | ZeroPageX | ZeroPageY | IndirectX | IndirectY
        | IndirectYWithCross | Relative => 2,

        // 3 bytes: opcode + 2 operand bytes
        Absolute | AbsoluteX | AbsoluteXWithCross | AbsoluteY | AbsoluteYWithCross | Indirect => 3,
    }
}
