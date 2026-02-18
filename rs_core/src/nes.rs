#[cfg(debug_assertions)]
use crate::cpu::{CpuObserver, CpuSnapshot};
use crate::{bus::Bus, cart::Cartridge, cpu::Cpu};
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

trait Clock {}

struct NesClock {}

impl Clock for NesClock {}

struct PalClock {}

impl Clock for PalClock {}

#[pyo3::pyclass(unsendable)]
pub struct Nes {
    bus: Bus,
    cpu: Cpu,
}

#[pyo3::pymethods]
impl Nes {
    #[new]
    pub fn new() -> Self {
        let mut nes = Nes {
            bus: Bus::new(),
            cpu: Cpu::new(),
        };

        #[cfg(debug_assertions)]
        nes.cpu.register_observer(Box::new(CpuLogger::new()));

        nes
    }

    fn load(&mut self, path: PathBuf) -> PyResult<()> {
        self.load_cart(&path)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    pub fn run_test(&mut self) {
        // List of test ROMs in order

        let tests = [
            "instr_test-v5/01-basics.nes",
            "instr_test-v5/02-implied.nes",
            "instr_test-v5/03-immediate.nes",
            "instr_test-v5/04-zero_page.nes",
            "instr_test-v5/05-zp_xy.nes",
            "instr_test-v5/06-absolute.nes",
            "instr_test-v5/07-abs_xy.nes",
            "instr_test-v5/08-ind_x.nes",
            "instr_test-v5/09-ind_y.nes",
            "instr_test-v5/10-branches.nes",
            "instr_test-v5/11-stack.nes",
            "instr_test-v5/12-jmp_jsr.nes",
            "instr_test-v5/13-rts.nes",
            "instr_test-v5/14-rti.nes",
            "instr_test-v5/15-brk.nes",
            "instr_test-v5/16-special.nes",
        ];

        for test_file in tests.iter() {
            self.cpu = Cpu::new();
            self.bus = Bus::new();

            //self.cpu.register_observer(Box::new(CpuLogger::new()));
            println!("=== Running test: {} ===", test_file);

            // Load the test cartridge
            let res = self.load_cart(test_file);
            match res {
                Ok(_) => println!("Loaded cartridge successfully"),
                Err(err) => {
                    eprintln!("Error loading cartridge {}: {}", test_file, err);
                    continue; // skip this test
                }
            }

            // Reset CPU and start test
            self.cpu.reset(&mut self.bus);

            let mut started = false;

            loop {
                if self.cpu.is_unoff() {
                    println!("Reached unoffical");
                    break;
                }
                self.step();

                // Read test status from $6000
                let status = self.bus.read(0x6000);

                match status {
                    0x80 if !started => {
                        // Test has just started
                        started = true;
                        println!("Test {} started", test_file);
                    }
                    0x81 => {
                        // Special case: CPU reset
                        println!("Resetting CPU for {}", test_file);
                        self.cpu.reset(&mut self.bus);
                    }
                    0x00..=0x7F => {
                        // Test may have finished, check started or PC separately
                        if started || self.cpu.pc == 0xFFFE {
                            println!("Finished test {} with value: {}", test_file, status);
                            break;
                        }
                    }
                    _ => {
                        if self.cpu.pc == 0xFFFE {
                            println!("Finished test {} with value: {}", test_file, status);
                            break;
                        }
                        // Test is running but no special action
                        // Optionally, you could log status for debugging
                        // println!("Running... status: {}", status);
                    }
                }
                let de = self.bus.read(0x6001);
                let b0 = self.bus.read(0x6002);
                // Optionally fetch null-terminated test result string at $6004+
            }

            // Optionally fetch null-terminated test result string at $6004+
            self.fetch_test_result();
            println!("----------------------------------------");
        }
    }

    pub fn fetch_test_result(&self) {
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

        // Convert to string if it's ASCII
        if let Ok(text) = std::str::from_utf8(&data) {
            println!("Read string: {}", text);
        } else {
        }
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

#[cfg(debug_assertions)]
#[derive(Clone)]
pub struct CpuLogger {
    snapshots: Vec<CpuSnapshot>,
    cycles_taken: u64,
}

#[cfg(debug_assertions)]
impl CpuLogger {
    pub fn new() -> Self {
        CpuLogger {
            snapshots: Vec::new(),
            cycles_taken: 0,
        }
    }
    pub fn log(&self, snap: &CpuSnapshot) {
        let bytes_str = match snap.opcode_bytes.len() {
            1 => format!("{:02X}      ", snap.opcode_bytes[0]),
            2 => format!(
                "{:02X} {:02X}   ",
                snap.opcode_bytes[0], snap.opcode_bytes[1]
            ),
            3 => format!(
                "{:02X} {:02X} {:02X}",
                snap.opcode_bytes[0], snap.opcode_bytes[1], snap.opcode_bytes[2]
            ),
            _ => format!("{:02X}      ", snap.opcode_bytes[0]),
        };

        let line = format!(
            "{:04X}  {}  {}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            snap.pc,
            bytes_str,
            snap.current_op,
            snap.a,
            snap.x,
            snap.y,
            snap.status.bits(),
            snap.sp,
            snap.cycles_taken,
        );
        println!("{}", line);
    }
}

#[cfg(debug_assertions)]
impl CpuObserver for CpuLogger {
    fn update(&mut self, snap: CpuSnapshot) {
        self.cycles_taken += snap.cycles_taken as u64;

        self.log(&snap);
    }
}
