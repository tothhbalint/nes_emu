from pathlib import Path

from emulator import Nes

def test_entry_point():
    n = Nes()
    n.load("../../crates/core/tests/roms/nestest.nes")
    for _ in range(100):
        n.step()
    s = n.snap()

    assert s.pc != 0, "PC should advance"

def test_inst_testv5():
    n = Nes()
    folder_name="../../crates/core/tests/roms/instr_test-v5"
    folder = Path(folder_name)

    for path in sorted(folder.iterdir()):
        if path.is_file() and path.suffix.lower() == ".nes":
            n.load(str(path))


if __name__ == "__main__":
    test_inst_testv5()
    print("end")
