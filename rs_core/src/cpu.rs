mod op;

use std::ops::RangeBounds;

use crate::{bus::Bus, cpu::op::make_op_table};
use bitflags::bitflags;
use bitflags::Flags;
use op::addressing;
use op::Op;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Status: u8 {
        const C = 1;      // Carry
        const Z = 1 << 1; // Zero
        const I = 1 << 2; // Disable Interrupt
        const D = 1 << 3; // Decimal Mode
        const B = 1 << 4; // Break
        const U = 1 << 5; // Unused
        const V = 1 << 6; // Overflow
        const N = 1 << 7; // Negative
    }
}

const OP_LOOKUP: [Op; 256] = make_op_table();

struct OpState {}

pub struct Cpu {
    //Registers
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    status: Status,
    //Timing
    cycles_left: u8,
    cycles: u64,
    #[cfg(debug_assertions)]
    observers: Vec<Box<dyn CpuObserver>>,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xC000,
            sp: 0xFD,
            status: Status::U.union(Status::I),
            cycles: 7,
            cycles_left: 0,
            #[cfg(debug_assertions)]
            observers: Vec::new(),
        }
    }
    #[inline]
    fn push_stack(&mut self, bus: &mut Bus, data: u8) {
        bus.write_byte(0x0100 | self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    #[inline]
    fn pop_stack(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let data = bus.read(0x0100 | self.sp as u16);

        data
    }

    #[inline]
    fn fetch_byte(&self, bus: &mut Bus, addr: u16) -> u8 {
        bus.read(addr)
    }

    #[inline]
    fn fetch_word(&self, bus: &mut Bus, addr: u16) -> u16 {
        let hi = self.fetch_byte(bus, addr);
        let lo = self.fetch_byte(bus, addr + 1);

        (lo as u16) << 8 | hi as u16
    }

    fn set_zn(&mut self, value: u8) {
        // Z flag: set if value is zero
        self.status.set(Status::Z, value == 0);

        // N flag: set if bit 7 is set (value & 0x80 != 0)
        self.status.set(Status::N, value & 0x80 != 0);
    }

    fn set_zvn(&mut self, value: u8) {
        // Z flag: set if value is zero
        self.status.set(Status::Z, value == 0);

        // N flag: set if bit 7 is set (value & 0x80 != 0)
        self.status.set(Status::N, value & 0x80 != 0);

        // V flag: set if bit 6 is set (value & 0x40 != 0)
        self.status.set(Status::V, value & 0x40 != 0);
    }

    fn page_cross_add(&self, lhs: u16, rhs: u16) -> (u16, bool) {
        let res = lhs.wrapping_add(rhs);
        let carry = (res & 0xFF00) != (lhs & 0xFF00);

        (res, carry)
    }

    pub fn step(&mut self, bus: &mut Bus) {
        if self.pc == 0xFFFF {
            return;
        }

        if self.cycles_left != 0 {
            self.cycles += 1;
            self.cycles_left -= 1;
        } else {
            let opcode = self.fetch_byte(bus, self.pc);

            let op = OP_LOOKUP[opcode as usize];

            let (operand, offset, page_cross): (u16, u8, bool) = match op.addr_mode {
                addressing::Mode::Accumulator | addressing::Mode::Implied => (0x0000, 1, false),
                addressing::Mode::Immediate | addressing::Mode::Relative => {
                    let addr = self.pc + 1;
                    (addr, 2, false)
                }
                addressing::Mode::Absolute => {
                    let addr = self.fetch_word(bus, self.pc + 1);
                    (addr, 3, false)
                }
                addressing::Mode::AbsoluteX | addressing::Mode::AbsoluteXWithCross => {
                    let addr = self.fetch_word(bus, self.pc + 1);
                    let (res, carry) = self.page_cross_add(addr, self.x as u16);
                    (res, 3, carry)
                }
                addressing::Mode::AbsoluteY | addressing::Mode::AbsoluteYWithCross => {
                    let addr = self.fetch_word(bus, self.pc + 1);
                    let (res, carry) = self.page_cross_add(addr, self.y as u16);
                    (res, 3, carry)
                }
                addressing::Mode::ZeroPage => {
                    let addr = self.fetch_byte(bus, self.pc + 1) as u16;
                    (addr, 2, false)
                }
                addressing::Mode::ZeroPageX => {
                    let mut addr = self.fetch_byte(bus, self.pc + 1);
                    addr = addr.wrapping_add(self.x);
                    (addr as u16, 2, false)
                }
                addressing::Mode::ZeroPageY => {
                    let mut addr = self.fetch_byte(bus, self.pc + 1);
                    addr = addr.wrapping_add(self.y);
                    (addr as u16, 2, false)
                }
                addressing::Mode::Indirect => {
                    let ptr_addr = self.fetch_word(bus, self.pc + 1);

                    let final_addr = if ptr_addr & 0xFF == 0xFF {
                        // Page boundary bug: wrap within page
                        let lo = self.fetch_byte(bus, ptr_addr);
                        let hi = self.fetch_byte(bus, ptr_addr & 0xFF00); // Wraps!
                        ((hi as u16) << 8) | (lo as u16)
                    } else {
                        // Normal case
                        self.fetch_word(bus, ptr_addr)
                    };

                    (final_addr, 3, false)
                }
                /*
                 * val = PEEK(PEEK((arg + X) % 256) + PEEK((arg + X + 1) % 256) * 256)
                 */
                addressing::Mode::IndirectX => {
                    let arg = self.fetch_byte(bus, self.pc + 1);
                    let ptr_addr = arg.wrapping_add(self.x);

                    // Read the two bytes from zero page to form the pointer
                    let lo = self.fetch_byte(bus, ptr_addr as u16) as u16;
                    let hi = self.fetch_byte(bus, ptr_addr.wrapping_add(1) as u16) as u16;

                    let addr_ptr = lo | (hi << 8);
                    (addr_ptr, 2, false)
                }
                /*
                val = PEEK(PEEK(arg) + PEEK((arg + 1) % 256) * 256 + Y)
                */
                addressing::Mode::IndirectY | addressing::Mode::IndirectYWithCross => {
                    let arg = self.fetch_byte(bus, self.pc + 1);
                    let lo = self.fetch_byte(bus, arg as u16) as u16;
                    let hi = self.fetch_byte(bus, arg.wrapping_add(1) as u16) as u16;

                    let addr_ptr = (lo | (hi << 8));
                    let (res, carry) = self.page_cross_add(addr_ptr, self.y as u16);

                    (res as u16, 2, carry)
                }
                _ => (0xBEEF, 0, false),
            };

            #[cfg(debug_assertions)]
            self.take_snapshot(op, bus, opcode as usize, offset);

            self.cycles_left = op.base_cycles;

            if page_cross
                && matches!(
                    op.addr_mode,
                    addressing::Mode::AbsoluteXWithCross
                        | addressing::Mode::AbsoluteYWithCross
                        | addressing::Mode::IndirectYWithCross
                )
            {
                self.cycles_left += 1;
            }

            self.pc = self.pc.wrapping_add(offset as u16);
            (op.exec)(self, bus, operand);
        }
    }

    /* ----------------------
     * INSTRUCTIONS
     * ----------------------
     */
    fn nop(&mut self, _bus: &mut Bus, _operand: u16) {}

    #[inline]
    fn brk(&mut self, bus: &mut Bus, _operand: u16) {
        let hi = (self.pc >> 8) as u8;
        let lo = (self.pc & 0xFF) as u8;
        self.push_stack(bus, hi);
        self.push_stack(bus, lo);
        self.push_stack(bus, self.status.bits() | 0x10);
        self.pc = 0xFFFE;

        self.status.set(Status::I, true);
    }

    #[inline]
    fn adc(&mut self, bus: &mut Bus, operand: u16) {
        let value = self.fetch_byte(bus, operand);

        let carry = self.status.contains(Status::C) as u8;

        let (sum1, c1) = self.a.overflowing_add(value);
        let (sum2, c2) = sum1.overflowing_add(carry);

        self.status.set(Status::C, c1 || c2);
        self.status
            .set(Status::V, (self.a ^ sum2) & (value ^ sum2) & 0x80 != 0);

        self.a = sum2;
        self.set_zn(self.a);
    }
    #[inline]
    fn and(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);
        self.a = self.a & memory;

        self.set_zn(self.a);
    }

    fn asl_a(&mut self, _bus: &mut Bus, _operand: u16) {
        let mut memory = self.a;
        self.status.set(Status::C, memory & 0x80 != 0);
        memory = memory << 1;

        self.a = memory;
        self.set_zn(self.a);
    }
    #[inline]
    fn asl(&mut self, bus: &mut Bus, operand: u16) {
        let mut memory = self.fetch_byte(bus, operand);
        self.status.set(Status::C, memory & 0x80 != 0);
        memory = memory << 1;

        bus.write_byte(operand, memory);
        self.set_zn(memory);
    }

    #[inline]
    fn bcc(&mut self, bus: &mut Bus, operand: u16) {
        if !self.status.contains(Status::C) {
            self.cycles_left += 1;
            let memory = self.fetch_byte(bus, operand) as i8;
            self.pc = self.pc.wrapping_add(memory as u16); // Only adding one as pc is already incremented
        }
    }
    #[inline]
    fn bcs(&mut self, bus: &mut Bus, operand: u16) {
        if self.status.contains(Status::C) {
            self.cycles_left += 1;
            let memory = self.fetch_byte(bus, operand) as i8;
            self.pc = self.pc.wrapping_add(memory as u16); // Only adding one as pc is already incremented
        }
    }
    #[inline]
    fn beq(&mut self, bus: &mut Bus, operand: u16) {
        if self.status.contains(Status::Z) {
            self.cycles_left += 1;
            let memory = self.fetch_byte(bus, operand) as i8;
            self.pc = self.pc.wrapping_add(memory as u16); // Only adding one as pc is already incremented
        }
    }
    #[inline]
    fn bit(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);

        self.status.set(Status::Z, self.a & memory == 0);
        self.status.set(Status::V, memory & 0x40 != 0);
        self.status.set(Status::N, memory & 0x80 != 0);
    }

    #[inline]
    fn bmi(&mut self, bus: &mut Bus, operand: u16) {
        if self.status.contains(Status::N) {
            self.cycles_left += 1;
            let memory = self.fetch_byte(bus, operand) as i8;
            self.pc = self.pc.wrapping_add(memory as u16); // Only adding one as pc is already incremented
        }
    }
    #[inline]
    fn bne(&mut self, bus: &mut Bus, operand: u16) {
        if !self.status.contains(Status::Z) {
            self.cycles_left += 1;
            let memory = self.fetch_byte(bus, operand) as i8;
            self.pc = self.pc.wrapping_add(memory as u16); // Only adding one as pc is already incremented
        }
    }

    #[inline]
    fn bpl(&mut self, bus: &mut Bus, operand: u16) {
        if !self.status.contains(Status::N) {
            self.cycles_left += 1;
            let memory = self.fetch_byte(bus, operand) as i8;
            self.pc = self.pc.wrapping_add(memory as u16); // Only adding one as pc is already incremented
        }
    }

    #[inline]
    fn bvc(&mut self, bus: &mut Bus, operand: u16) {
        if !self.status.contains(Status::V) {
            self.cycles_left += 1;
            let memory = self.fetch_byte(bus, operand) as i8;
            self.pc = self.pc.wrapping_add(memory as u16); // Only adding one as pc is already incremented
        }
    }
    #[inline]
    fn bvs(&mut self, bus: &mut Bus, operand: u16) {
        if self.status.contains(Status::V) {
            self.cycles_left += 1;
            let memory = self.fetch_byte(bus, operand) as i8;
            self.pc = self.pc.wrapping_add(memory as u16); // Only adding one as pc is already incremented
        }
    }
    #[inline]
    fn cmp(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);
        let val = self.a.wrapping_sub(memory);
        self.status.set(Status::C, self.a >= memory);
        self.set_zn(val);
    }

    #[inline]
    fn cpx(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);
        let val = self.x.wrapping_sub(memory);
        self.status.set(Status::C, self.x >= memory);
        self.set_zn(val);
    }
    #[inline]
    fn cpy(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);
        let val = self.y.wrapping_sub(memory);
        self.status.set(Status::C, self.y >= memory);
        self.set_zn(val);
    }
    #[inline]
    fn clc(&mut self, bus: &mut Bus, operand: u16) {
        self.status.remove(Status::C);
    }
    #[inline]
    fn cld(&mut self, bus: &mut Bus, operand: u16) {
        self.status.remove(Status::D);
    }
    #[inline]
    fn cli(&mut self, bus: &mut Bus, operand: u16) {
        self.status.remove(Status::I);
    }
    #[inline]
    fn clv(&mut self, bus: &mut Bus, operand: u16) {
        self.status.remove(Status::V);
    }
    #[inline]
    fn dec(&mut self, bus: &mut Bus, operand: u16) {
        let mut memory = self.fetch_byte(bus, operand);

        memory = memory.wrapping_sub(1);
        bus.write_byte(operand, memory);

        self.set_zn(memory);
    }
    #[inline]
    fn dex(&mut self, bus: &mut Bus, operand: u16) {
        self.x = self.x.wrapping_sub(1);
        self.set_zn(self.x);
    }
    #[inline]
    fn dey(&mut self, bus: &mut Bus, operand: u16) {
        self.y = self.y.wrapping_sub(1);
        self.set_zn(self.y);
    }
    #[inline]
    fn eor(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);
        self.a = self.a ^ memory;

        self.set_zn(self.a);
    }

    #[inline]
    fn inc(&mut self, bus: &mut Bus, operand: u16) {
        let mut memory = self.fetch_byte(bus, operand);
        memory = memory.wrapping_add(1);
        bus.write_byte(operand, memory);

        self.set_zn(memory);
    }
    #[inline]
    fn inx(&mut self, bus: &mut Bus, operand: u16) {
        self.x = self.x.wrapping_add(1);
        self.set_zn(self.x);
    }
    #[inline]
    fn iny(&mut self, bus: &mut Bus, operand: u16) {
        self.y = self.y.wrapping_add(1);
        self.set_zn(self.y);
    }

    #[inline]
    fn jmp(&mut self, bus: &mut Bus, operand: u16) {
        self.pc = operand;
    }
    #[inline]
    fn jsr(&mut self, bus: &mut Bus, operand: u16) {
        let addr = self.pc - 1;
        let hi = (addr >> 8) as u8;
        let lo = (addr & 0xFF) as u8;
        self.push_stack(bus, hi);
        self.push_stack(bus, lo);
        self.pc = operand;
    }
    #[inline]
    fn lda(&mut self, bus: &mut Bus, operand: u16) {
        self.a = self.fetch_byte(bus, operand);
        self.set_zn(self.a);
    }
    #[inline]
    fn ldx(&mut self, bus: &mut Bus, operand: u16) {
        self.x = self.fetch_byte(bus, operand);
        self.set_zn(self.x);
    }
    #[inline]
    fn ldy(&mut self, bus: &mut Bus, operand: u16) {
        self.y = self.fetch_byte(bus, operand);
        self.set_zn(self.y);
    }

    fn lsr_a(&mut self, _bus: &mut Bus, _operand: u16) {
        let mut memory = self.a;
        self.status.set(Status::C, memory & 0x1 != 0);

        memory = memory >> 1;
        self.status.set(Status::Z, memory == 0);
        self.status.remove(Status::N);

        self.a = memory;
    }
    #[inline]
    fn lsr(&mut self, bus: &mut Bus, operand: u16) {
        let mut memory = self.fetch_byte(bus, operand);
        self.status.set(Status::C, memory & 0x1 != 0);

        memory = memory >> 1;
        self.status.set(Status::Z, memory == 0);
        self.status.remove(Status::N);

        bus.write_byte(operand, memory);
    }

    #[inline]
    fn ora(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);
        self.a = self.a | memory;
        self.set_zn(self.a);
    }
    #[inline]
    fn pha(&mut self, bus: &mut Bus, operand: u16) {
        self.push_stack(bus, self.a);
    }
    #[inline]
    fn php(&mut self, bus: &mut Bus, operand: u16) {
        let mut flags = self.status.bits();

        flags |= 0x10;
        flags |= 0x20;

        self.push_stack(bus, flags);
    }
    #[inline]
    fn pla(&mut self, bus: &mut Bus, operand: u16) {
        let val = self.pop_stack(bus);
        self.a = val;

        self.set_zn(self.a);
    }
    #[inline]
    fn plp(&mut self, bus: &mut Bus, operand: u16) {
        let mut pulled = self.pop_stack(bus);
        self.status = Status::from_bits(pulled & 0xEF | 0x20).unwrap();
    }

    #[inline]
    fn rol_a(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.a;

        let result = (memory << 1) | self.status.contains(Status::C) as u8;
        self.status.set(Status::C, memory & 0x80 != 0);
        self.set_zn(result);

        self.a = result;
    }

    #[inline]
    fn rol(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);

        let result = (memory << 1) | self.status.contains(Status::C) as u8;
        self.status.set(Status::C, memory & 0x80 != 0);
        self.set_zn(result);

        bus.write_byte(operand, result);
    }

    #[inline]
    fn ror_a(&mut self, bus: &mut Bus, operand: u16) {
        let mut memory = self.a;

        memory = (memory >> 1) | ((self.status.contains(Status::C) as u8) << 7);
        self.set_zn(memory);
        self.status.set(Status::C, self.a & 0x1 != 0);

        self.a = memory;
    }

    #[inline]
    fn ror(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);

        let result = (memory >> 1) | ((self.status.contains(Status::C) as u8) << 7);
        self.set_zn(result);
        self.status.set(Status::C, memory & 0x1 != 0);

        bus.write_byte(operand, result);
    }

    #[inline]
    fn rti(&mut self, bus: &mut Bus, operand: u16) {
        let mut pulled = self.pop_stack(bus);
        self.status = Status::from_bits(pulled & 0xEF | 0x20).unwrap();
        let lo = self.pop_stack(bus);
        let hi = self.pop_stack(bus);

        self.pc = (hi as u16) << 8 | (lo as u16);
    }
    #[inline]
    fn rts(&mut self, bus: &mut Bus, operand: u16) {
        let lo = self.pop_stack(bus);
        let hi = self.pop_stack(bus);

        self.pc = (hi as u16) << 8 | (lo as u16);
        self.pc += 1;
    }
    #[inline]
    fn sbc(&mut self, bus: &mut Bus, operand: u16) {
        let memory = self.fetch_byte(bus, operand);
        let carry = self.status.contains(Status::C) as u8;
        let value = memory ^ 0xFF;

        let (sum1, c1) = self.a.overflowing_add(value);
        let (sum2, c2) = sum1.overflowing_add(carry);

        self.status.set(Status::C, c1 || c2);
        self.status
            .set(Status::V, (self.a ^ sum2) & (value ^ sum2) & 0x80 != 0);

        self.a = sum2;
        self.set_zn(self.a);
    }
    #[inline]
    fn sec(&mut self, bus: &mut Bus, operand: u16) {
        self.status.set(Status::C, true);
    }
    #[inline]
    fn sed(&mut self, bus: &mut Bus, operand: u16) {
        self.status.set(Status::D, true);
    }
    #[inline]
    fn sei(&mut self, bus: &mut Bus, operand: u16) {
        self.status.set(Status::I, true);
    }
    #[inline]
    fn sta(&mut self, bus: &mut Bus, operand: u16) {
        bus.write_byte(operand, self.a);
    }
    #[inline]
    fn stx(&mut self, bus: &mut Bus, operand: u16) {
        bus.write_byte(operand, self.x);
    }
    #[inline]
    fn sty(&mut self, bus: &mut Bus, operand: u16) {
        bus.write_byte(operand, self.y);
    }
    #[inline]
    fn tax(&mut self, bus: &mut Bus, operand: u16) {
        self.x = self.a;
        self.set_zn(self.x);
    }
    #[inline]
    fn tay(&mut self, bus: &mut Bus, operand: u16) {
        self.y = self.a;
        self.set_zn(self.y);
    }
    #[inline]
    fn tsx(&mut self, bus: &mut Bus, operand: u16) {
        self.x = self.sp;
        self.set_zn(self.x);
    }
    #[inline]
    fn txa(&mut self, bus: &mut Bus, operand: u16) {
        self.a = self.x;
        self.set_zn(self.a);
    }
    #[inline]
    fn txs(&mut self, bus: &mut Bus, operand: u16) {
        self.sp = self.x;
    }
    #[inline]
    fn tya(&mut self, _bus: &mut Bus, _operand: u16) {
        self.a = self.y;
        self.set_zn(self.a);
    }
}

/* -------
 *  Debug
 * -------
 */
#[cfg(debug_assertions)]
impl Cpu {
    pub fn register_observer(&mut self, obs: Box<dyn CpuObserver>) {
        self.observers.push(obs);
    }

    fn take_snapshot(&mut self, op: Op, bus: &mut Bus, opcode: usize, offset: u8) {
        let opcode_bytes: Vec<u8> = match offset {
            1 => vec![opcode as u8],
            2 => vec![opcode as u8, self.fetch_byte(bus, self.pc + 1)],
            3 => vec![
                opcode as u8,
                self.fetch_byte(bus, self.pc + 1),
                self.fetch_byte(bus, self.pc + 2),
            ],
            _ => vec![opcode as u8],
        };

        let snap: CpuSnapshot = CpuSnapshot {
            a: (self.a),
            x: (self.x),
            y: (self.y),
            pc: (self.pc),
            sp: (self.sp),
            status: (self.status),
            cycles_taken: (self.cycles),
            current_op: op.name,
            opcode_bytes: opcode_bytes,
        };
        for observer in &mut self.observers {
            observer.update(snap.clone());
        }
    }
}

#[cfg(debug_assertions)]
#[derive(Clone)]
pub struct CpuSnapshot {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub status: Status,
    //Timing
    pub cycles_taken: u64,
    pub current_op: &'static str,
    pub opcode_bytes: Vec<u8>,
}

#[cfg(debug_assertions)]
pub trait CpuObserver {
    fn update(&mut self, snap: CpuSnapshot);
}
