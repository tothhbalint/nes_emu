use emulator_core::nes::Nes;
use pyo3::prelude::*;

#[pyclass(name = "CpuSnapshot", get_all, skip_from_py_object)]
#[derive(Clone)]
pub struct PyCpuSnapshot {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub status: u8,
}

#[pyclass(name = "DisasmEntry", get_all, skip_from_py_object)]
#[derive(Clone)]
pub struct PyDisasmEntry {
    pub pc: u16,
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
        Ok(self.inner.load_cart(path)?)
    }

    fn step(&mut self) {
        self.inner.step();
    }

    fn snap(&mut self) -> PyCpuSnapshot {
        let s = self.inner.snap();
        PyCpuSnapshot {
            a: s.a,
            x: s.x,
            y: s.y,
            pc: s.pc,
            sp: s.sp,
            status: s.status.bits(),
        }
    }

    /// Decode `count` instructions forward from `start` in one FFI call.
    fn disassemble_forward(&self, start: u16, count: usize) -> Vec<PyDisasmEntry> {
        self.inner
            .disassemble_forward(start, count)
            .into_iter()
            .map(|e| PyDisasmEntry {
                pc: e.pc,
                opcode: e.opcode[..e.len as usize].to_vec(),
                len: e.len,
                mnemonic: e.mnemonic.to_string(),
            })
            .collect()
    }

    fn run_until_breakpoint(&mut self, addr: u16) {
        self.inner.run_until_breakpoint(addr);
    }
}

#[pymodule]
fn _emulator(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNes>()?;
    m.add_class::<PyCpuSnapshot>()?;
    m.add_class::<PyDisasmEntry>()?;
    Ok(())
}
