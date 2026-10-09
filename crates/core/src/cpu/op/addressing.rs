#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Implied,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Accumulator,
    Relative,
    AbsoluteXWithCross,
    AbsoluteYWithCross,
    IndirectYWithCross,
}
