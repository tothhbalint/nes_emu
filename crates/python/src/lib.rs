use emulator_core::nes::Nes;
use pyo3::prelude::*; // core type, plain Rust

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
    fn load(&mut self, path: std::path::PathBuf) -> PyResult<()> { /* map io::Error */
    }
    fn run_frame(&mut self) {
        self.inner.run_frame();
    }
    fn step(&mut self) {
        self.inner.step();
    }
    fn tick(&mut self) {
        self.inner.tick();
    }
    #[getter]
    fn pc(&self) -> u16 {
        self.inner.pc()
    }
    fn disassemble(&self, addr: u16) -> String {
        self.inner.disassemble(addr)
    }
    // memory r/w, breakpoints, framebuffer, set_buttons...
}

#[pymodule]
fn _emulator(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNes>()?;
    Ok(())
}
