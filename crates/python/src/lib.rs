use emulator_core::{cpu::CpuSnapshot, nes::Nes};
use pyo3::prelude::*; // core type, plain Rust

#[pyclass(name = "CpuSnapshot", get_all)]
#[derive(Clone)]
pub struct PyCpuSnapshot {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub status: u8,
    pub opcode: Vec<u8>,
    pub len: u8,
    pub mnemonic: String,
}

#[pyclass(name = "Nes", unsendable)]
struct PyNes {
    inner: Nes,
}

#[pymethods]
impl PyNes {
    #[new]
    fn new() -> Self {
        PyNes { inner: Nes::new() }
    }
    fn load(&mut self, path: std::path::PathBuf) -> PyResult<()> {
        return Ok(self.inner.load_cart(path)?);
    }

    fn step(&mut self) {
        self.inner.step();
    }

    fn snap(&mut self) -> PyCpuSnapshot {
        let s = self.inner.snap();

        let (opcode, len, mnemonic) = (
            s.instr.opcode.to_vec(),
            s.instr.len,
            s.instr.mnemonic.to_string(),
        );

        PyCpuSnapshot {
            a: s.a,
            x: s.x,
            y: s.y,
            pc: s.pc,
            sp: s.sp,
            status: s.status.bits(),
            opcode,
            len,
            mnemonic,
        }
    }

    fn run_until_breakpoint(&mut self, addr: u16) {
        self.inner.run_until_breakpoint(addr);
    }

    // fn run_frame(&mut self) {
    //     self.inner.run_frame();
    // }
    // fn step(&mut self) {
    //     self.inner.step();
    // }
    // fn tick(&mut self) {
    //     self.inner.tick();
    // }
    // #[getter]
    // fn pc(&self) -> u16 {
    //     self.inner.pc()
    // }
    // fn disassemble(&self, addr: u16) -> String {
    //     self.inner.disassemble(addr)
    // }
    // memory r/w, breakpoints, framebuffer, set_buttons...
}

#[pymodule]
fn _emulator(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNes>()?;
    m.add_class::<PyCpuSnapshot>()?;
    Ok(())
}
