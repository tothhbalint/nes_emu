# NES Emulator

A Rust NES emulator with a core library, a desktop app, and a Python bridge.

Most of the UI was written using LLMs for quick prototyping.

## Crates

| Crate | Package | Purpose |
|---|---|---|
| `crates/core` | `emulator-core` | CPU, bus, cartridge, PPU, APU |
| `crates/app` | `emulator-app` | Desktop binary |
| `crates/python` | `emulator-python` | Python extension (`_emulator`) |
| `crates/bench` | `test_bench` | Criterion benchmarks |


## Build

```sh
cargo build
cargo run -p emulator-app
cargo test -p emulator-core --features debug
```

Python extension:

```sh
python3 -m venv .venv && source .venv/bin/activate
pip install maturin
cd crates/python && maturin develop --release
```

## Status

- CPU, bus, and cartridge are implemented; `nestest` and `instr_test-v5` are the end-to-end tests.
- PPU is rendering an image mostly correctly
    - The timing is done scanline by scanline so it's not a 100% accurate
    - It can render pacman 
- APU is a stub; mappers are not implemented.
- The debug API is partially wired; the Python bridge is a work in progress.
