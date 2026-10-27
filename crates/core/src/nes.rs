#[cfg(feature = "debug")]
use crate::cpu::{CpuSnapshot, DisasmEntry};
use crate::{bus::Bus, cart::Cartridge, cpu::Cpu};
#[cfg(feature = "debug")]
use std::io::{BufWriter, Write};
use std::path::Path;

trait Clock {}

struct NesClock {}

impl Clock for NesClock {}

struct PalClock {}

impl Clock for PalClock {}

pub struct Nes {
    bus: Bus,
    cpu: Cpu,
}

impl Nes {
    pub fn new() -> Self {
        Nes {
            bus: Bus::new(),
            cpu: Cpu::new(),
        }
    }

    pub fn step(&mut self) {
        self.cpu.tick(&mut self.bus);
    }

    pub fn fetch_test_result(&self) -> Vec<u8> {
        let mut addr: u16 = 0x6004;
        let mut data: Vec<u8> = Vec::new();

        loop {
            let byte = self.bus.read(addr);
            if byte == 0x00 {
                break;
            }
            data.push(byte);
            addr = addr.wrapping_add(1);
        }

        data
    }
}

impl Nes {
    pub fn load_cart<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let rom = std::fs::read(path)?;
        let cart = Cartridge::from_bytes(&rom)?;
        self.bus.insert_cart(cart);
        Ok(())
    }
}

pub struct TestResult {
    pub status: u8,
    pub output: Vec<u8>,
}

impl Nes {
    pub fn run_nestest(&mut self) {
        self.load_cart("tests/roms/nestest.nes")
            .expect("failed to load nestest.nes");

        self.cpu.jump_to(0xC000);

        while !self.cpu.is_unoff() {
            // stop at the first unofficial opcode
            self.cpu.tick(&mut self.bus);
        }
    }

    pub fn run_test_rom(
        &mut self,
        rom: &'static str,
        max_cycles: usize,
        test_unoff: bool,
    ) -> TestResult {
        self.load_cart(rom).unwrap();

        self.cpu.reset(&mut self.bus);

        let mut started = false;
        let mut cycles = 0usize;

        loop {
            self.step();
            cycles += 1;

            let status = self.bus.read(0x6000);

            if !test_unoff && self.cpu.is_unoff() {
                return TestResult {
                    status: 0,
                    output: b"Reached unofficial".to_vec(),
                };
            }

            match status {
                0x80 if !started => started = true,    // test signalled "started"
                0x81 => self.cpu.reset(&mut self.bus), // test requested a reset
                0x00..=0x7F if started || self.cpu.pc() == 0xFFFE => {
                    // done; value = result (0 = pass)
                    return TestResult {
                        status,
                        output: self.fetch_test_result(),
                    };
                }
                _ => {}
            }

            if cycles >= max_cycles {
                // format a diagnostic: pc, next 3 opcode bytes, and registers
                let pc = self.cpu.pc();
                let b0 = self.bus.read(pc);
                let b1 = self.bus.read(pc.wrapping_add(1));
                let b2 = self.bus.read(pc.wrapping_add(2));
                let hang = format!(
                    "Test ROM timed out after {} cycles at {:04X}  {:02X} {:02X} {:02X}  {}",
                    cycles,
                    pc,
                    b0,
                    b1,
                    b2,
                    self.cpu.debug_registers(),
                );
                println!("{hang}");
                return TestResult {
                    status: 1,
                    output: hang.into_bytes(),
                };
            }
        }
    }
}

#[cfg(feature = "debug")]
pub struct CpuLogger {
    snapshots: Vec<CpuSnapshot>,
    cycles_taken: u64,
    writer: BufWriter<std::fs::File>,
}

#[cfg(feature = "debug")]
impl CpuLogger {
    pub fn new() -> Self {
        use std::fs::File;

        let file = File::create("log.log").expect("Failed to create log file at log.log");
        CpuLogger {
            snapshots: Vec::new(),
            cycles_taken: 0,
            writer: BufWriter::new(file),
        }
    }

    pub fn log(&mut self, snap: &CpuSnapshot) {
        // let bytes_str = match snap.len {
        //     1 => format!("{:02X}      ", snap.opcode_bytes[0]),
        //     2 => format!(
        //         "{:02X} {:02X}   ",
        //         snap.opcode_bytes[0], snap.opcode_bytes[1]
        //     ),
        //     3 => format!(
        //         "{:02X} {:02X} {:02X}",
        //         snap.opcode_bytes[0], snap.opcode_bytes[1], snap.opcode_bytes[2]
        //     ),
        //     _ => format!("{:02X}      ", snap.opcode_bytes[0]),
        // };

        // let line = format!(
        //     "{:04X}  {}  {}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
        //     snap.pc,
        //     bytes_str,
        //     snap.current_op,
        //     snap.a,
        //     snap.x,
        //     snap.y,
        //     snap.status.bits(),
        //     snap.sp,
        //     snap.cycles_taken,
        // );

        // let _ = writeln!(self.writer, "{}", line);
    }
}

#[cfg(feature = "debug")]
impl Nes {
    pub fn run_until_breakpoint(&mut self, addr: u16) {
        while self.cpu.pc() != addr {
            self.cpu.tick(&mut self.bus);
        }
    }

    pub fn restore_cpu(&mut self, snap: CpuSnapshot) {
        self.cpu.restore_snapshot(snap);
    }
    // TODO this should return MachineSnapshot eventually
    /*
     * Imagine something like this:
     * forall snapshottable: snapshotter.snapshot()
     * MachineSnapshot.add(Snapshot)
     */
    pub fn snap(&mut self) -> CpuSnapshot {
        self.cpu.take_snapshot()
    }

    pub fn disassemble_forward(&self, _start: u16, count: usize) -> Vec<DisasmEntry> {
        self.cpu.disassemble_forward(&self.bus, count)
    }
}

#[cfg(feature = "debug")]
pub struct CpuDebugger {
    snapshots: Vec<CpuSnapshot>,
    cycles_taken: u64,
}

#[cfg(feature = "debug")]
impl CpuDebugger {
    pub fn new() -> Self {
        CpuDebugger {
            snapshots: Vec::new(),
            cycles_taken: 0,
        }
    }

    #[inline]
    pub fn snap(&self) -> CpuSnapshot {
        self.snapshots[0]
    }
}
