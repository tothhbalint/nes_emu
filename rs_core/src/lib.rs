use pyo3::prelude::*;

use crate::nes::Nes;

pub mod bus;
pub mod cart;
pub mod cpu;
pub mod nes;

#[pymodule]
fn emulator(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Nes>()?;
    Ok(())
}
