use super::*;
use crate::cart::Cartridge;
use crate::ppu::Ppu;

const NMI_TARGET: u16 = 0x9000;

/// Minimal valid iNES image: 16 KB PRG, 8 KB CHR, no trainer.
/// The NMI vector at $FFFA points at `NMI_TARGET`.
fn bus_with_cart() -> Bus {
    let mut rom = vec![0u8; 16];
    rom[0..4].copy_from_slice(b"NES\x1A");
    rom[4] = 1; // 1 x 16 KB PRG
    rom[5] = 1; // 1 x 8 KB CHR
    rom.extend(std::iter::repeat(0u8).take(16 * 1024));
    rom.extend(std::iter::repeat(0u8).take(8 * 1024));

    // With a 16 KB PRG the file offset of $FFFA is 0x3FFA (the ROM is mirrored).
    rom[16 + 0x3FFA] = (NMI_TARGET & 0xFF) as u8;
    rom[16 + 0x3FFB] = (NMI_TARGET >> 8) as u8;

    let mut bus = Bus::new(Ppu::new());
    bus.insert_cart(Cartridge::from_bytes(&rom).expect("synthetic cart must parse"));
    bus
}

#[test]
fn nmi_pushes_state_and_jumps_to_vector() {
    let mut bus = bus_with_cart();
    let mut cpu = Cpu::new();
    cpu.jump_to(0x8000);

    let sp_before = cpu.sp;
    cpu.set_nmi();
    for _ in 0..7 {
        cpu.tick(&mut bus);
    }

    assert_eq!(cpu.pc(), NMI_TARGET, "PC should come from $FFFA/$FFFB");
    assert_eq!(cpu.sp, sp_before.wrapping_sub(3), "three bytes pushed");
    assert!(cpu.status.contains(Status::I), "NMI must set I");

    let pch = bus.read(0x0100 | sp_before as u16);
    let pcl = bus.read(0x0100 | (sp_before as u16).wrapping_sub(1));
    let p = bus.read(0x0100 | (sp_before as u16).wrapping_sub(2));

    assert_eq!(
        ((pch as u16) << 8) | pcl as u16,
        0x8000,
        "stacked PC must be the interrupted instruction"
    );
    assert_eq!(p & 0x10, 0x00, "B must be clear in an NMI frame");
    assert_eq!(p & 0x20, 0x20, "U must be set in an NMI frame");
}

#[test]
fn nmi_is_taken_with_interrupt_disable_set() {
    let mut bus = bus_with_cart();
    let mut cpu = Cpu::new();
    cpu.jump_to(0x8000);
    cpu.status.insert(Status::I); // set from reset anyway

    cpu.set_nmi();
    cpu.step(&mut bus);

    assert_eq!(cpu.pc(), NMI_TARGET, "I must not mask NMI");
}

#[test]
fn nmi_consumes_seven_cycles() {
    let mut bus = bus_with_cart();
    let mut cpu = Cpu::new();
    cpu.jump_to(0x8000);

    let before = cpu.cycles;
    cpu.set_nmi();
    cpu.step(&mut bus);

    assert_eq!(cpu.cycles - before, 7);
}

#[test]
fn no_nmi_leaves_the_cpu_running_normally() {
    let mut bus = bus_with_cart();
    let mut cpu = Cpu::new();
    cpu.jump_to(0x8000);

    let sp_before = cpu.sp;
    cpu.step(&mut bus); // opcode 0x00 = BRK

    assert_ne!(cpu.pc(), NMI_TARGET, "no NMI was requested");
    assert_eq!(cpu.sp, sp_before.wrapping_sub(3), "BRK pushes three bytes");
}

#[test]
fn nmi_pending_survives_until_serviced() {
    let mut bus = bus_with_cart();
    let mut cpu = Cpu::new();
    cpu.jump_to(0x8000);

    cpu.set_nmi();
    cpu.step(&mut bus); // services the NMI
    assert_eq!(cpu.pc(), NMI_TARGET);

    // the next step must execute the instruction at the vector, not re-enter
    // the handler: opcode 0x00 is BRK, whose vector at $FFFE is zero
    cpu.step(&mut bus);
    assert_eq!(
        cpu.pc(),
        0x0000,
        "second step did not execute the handler code"
    );
}
