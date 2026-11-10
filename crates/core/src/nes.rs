#[cfg(feature = "debug")]
use crate::cpu::{CpuSnapshot, DisasmEntry};
use crate::{
    bus::Bus,
    cart::Cartridge,
    cpu::Cpu,
    ppu::{self, Ppu},
};
#[cfg(feature = "debug")]
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct Nes {
    bus: Bus,
    cpu: Cpu,
}

impl Nes {
    pub fn new() -> Self {
        Nes {
            bus: Bus::new(Ppu::new()),
            cpu: Cpu::new(),
        }
    }

    pub fn run_frame(&mut self) {
        const CPU_CYCLES_PER_FRAME: usize = 29_780;
        const PPU_CYCLES_PER_FRAME: usize = 29_780 * 3;
        for i in 0..PPU_CYCLES_PER_FRAME {
            self.tick_by_ppu(i as u64);
        }
    }

    pub fn frame(&self) -> ppu::FrameBuffer {
        self.bus.ppu.render()
    }

    pub fn tick_by_ppu(&mut self, cycle: u64) {
        self.bus.ppu.tick();
        if self.bus.ppu.take_nmi() {
            self.cpu.set_nmi();
        }
        if cycle % 3 == 0 {
            self.cpu.tick(&mut self.bus);
        }
    }

    pub fn tick_by_cpu(&mut self) {
        self.bus.ppu.tick();
        self.bus.ppu.tick();
        self.bus.ppu.tick();
        if self.bus.ppu.take_nmi() {
            self.cpu.set_nmi();
        }
        self.cpu.tick(&mut self.bus);
    }

    pub fn fetch_test_result(&mut self) -> Vec<u8> {
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
    pub fn run_test_rom(&mut self, rom: &'static str, max_cycles: usize) -> TestResult {
        self.load_cart(rom).unwrap();

        self.cpu.reset(&mut self.bus);

        let mut started = false;
        let mut cycles = 0usize;

        loop {
            self.tick_by_cpu();
            cycles += 1;

            let status = self.bus.read(0x6000);

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
    /// Executes instructions until the PC lands on one of `breakpoints`,
    /// returning the address that was hit. An empty list returns immediately,
    /// since there is nothing to run towards.
    pub fn run_until_breakpoints(&mut self, breakpoints: &[u16]) -> Option<u16> {
        if breakpoints.is_empty() {
            return None;
        }

        loop {
            self.tick_by_cpu();
            let pc = self.cpu.pc();
            if breakpoints.contains(&pc) {
                return Some(pc);
            }
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

    pub fn disassemble_forward(&mut self, _start: u16, count: usize) -> Vec<DisasmEntry> {
        self.cpu.disassemble_forward(&mut self.bus, count)
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

/// Outcome of an automated nestest run.
pub struct NestestResult {
    /// Reached the final RTS at $C66E (i.e. the whole suite ran).
    pub finished: bool,
    /// Error code the ROM wrote to zero page. Both zero means success.
    pub error_lo: u8,
    pub error_hi: u8,
    pub steps: usize,
    pub pc: u16,
    pub registers: String,
}

impl NestestResult {
    pub fn passed(&self) -> bool {
        self.finished && self.error_lo == 0 && self.error_hi == 0
    }
}

impl Nes {
    /// Runs nestest in automated mode (starting at $C000) and reports what it
    /// wrote to zero page. Stops at the final RTS at $C66E, or on the step cap.
    pub fn run_nestest(&mut self) -> NestestResult {
        const MAX_STEPS: usize = 50000; // reference run is 8991 instructions

        self.load_cart("tests/roms/nestest.nes")
            .expect("failed to load nestest.nes");

        // Power-on state already matches the trace: pc=$C000, sp=$FD, P=$24.
        self.cpu.jump_to(0xC000);

        let mut steps = 0;
        while self.cpu.pc() != 0xC66E && steps < MAX_STEPS {
            self.tick_by_cpu();
            steps += 1;
        }

        NestestResult {
            finished: self.cpu.pc() == 0xC66E,
            error_lo: self.bus.read(0x02),
            error_hi: self.bus.read(0x03),
            steps,
            pc: self.cpu.pc(),
            registers: self.cpu.debug_registers(),
        }
    }
}
