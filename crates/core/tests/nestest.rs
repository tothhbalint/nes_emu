mod test {
    use emulator_core::nes;

    use std::fs;

    fn first_difference(expected: &str, actual: &str) -> String {
        let expected_content = fs::read_to_string(expected).unwrap();
        let actual_content = fs::read_to_string(actual).unwrap();
        for (i, (e_line, a_line)) in expected_content
            .lines()
            .zip(actual_content.lines())
            .enumerate()
        {
            if e_line != a_line {
                println!("Files differ at line {}:", i + 1);
                println!("EXPECTED: {}", e_line);
                println!("ACTUAL:   {}", a_line);
                return e_line.to_string(); // Stop at first difference
            }
        }
        // No differences found
        "".to_string()
    }

    #[test]
    pub fn nestest() {
        let mut nes = nes::Nes::new();

        nes.run_nestest();

        let diff = first_difference("tests/logs/nestest.log", "log.log");

        assert_eq!(
            diff,
            "C6BD  04 A9    *NOP  A:AA X:97 Y:4E P:EF SP:F9 CYC:14579"
        );

        std::fs::remove_file("log.log");
    }
}
