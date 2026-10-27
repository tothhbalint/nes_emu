mod test {
    use emulator_core::nes;

    macro_rules! rom_test {
        ($name:ident, $path:literal) => {
            #[test]
            fn $name() {
                let mut emu = nes::Nes::new();
                let result = emu.run_test_rom($path, 15_000_000, true);

                let output = String::from_utf8_lossy(&result.output);

                assert_eq!(result.status, 0, "ROM failed:\n{}", output);
            }
        };
    }

    rom_test!(basics, "tests/roms/instr_test-v5/01-basics.nes");
    rom_test!(implied, "tests/roms/instr_test-v5/02-implied.nes");
    rom_test!(immediate, "tests/roms/instr_test-v5/03-immediate.nes");
    rom_test!(zero_page, "tests/roms/instr_test-v5/04-zero_page.nes");
    rom_test!(zp_xy, "tests/roms/instr_test-v5/05-zp_xy.nes");
    rom_test!(absolute, "tests/roms/instr_test-v5/06-absolute.nes");
    rom_test!(abs_xy, "tests/roms/instr_test-v5/07-abs_xy.nes");
    rom_test!(ind_x, "tests/roms/instr_test-v5/08-ind_x.nes");
    rom_test!(ind_y, "tests/roms/instr_test-v5/09-ind_y.nes");
    rom_test!(branches, "tests/roms/instr_test-v5/10-branches.nes");
    rom_test!(stack, "tests/roms/instr_test-v5/11-stack.nes");
    rom_test!(jmp_jsr, "tests/roms/instr_test-v5/12-jmp_jsr.nes");
    rom_test!(rts, "tests/roms/instr_test-v5/13-rts.nes");
    rom_test!(rti, "tests/roms/instr_test-v5/14-rti.nes");
    rom_test!(brk, "tests/roms/instr_test-v5/15-brk.nes");
    rom_test!(special, "tests/roms/instr_test-v5/16-special.nes");
}
