mod test {
    use emulator_core::nes;

    #[test]
    fn nestest_status() {
        let mut nes = nes::Nes::new();
        let r = nes.run_nestest();

        assert!(
            r.passed(),
            "nestest failed after {} instructions at pc=${:04X} \
             (finished={}, error=${:02X}{:02X}, {})",
            r.steps,
            r.pc,
            r.finished,
            r.error_hi,
            r.error_lo,
            r.registers,
        );
    }
}
