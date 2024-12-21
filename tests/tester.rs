macro_rules! eval_and_assert {
    ($name:ident, $src:expr, $expected_out:expr, $expected_err:expr) => {
        #[test]
        fn $name() -> () {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();

            let interpreter =
                linefeed::interpreter::Interpreter::new_with_output(&mut stdout, &mut stderr);

            linefeed::run_with_interpreter(interpreter, $src);
            let stdout_str = std::str::from_utf8(&stdout).unwrap();
            let stderr_str = std::str::from_utf8(&stderr).unwrap();

            assert_eq_normalised("stderr", stderr_str, $expected_err);
            assert_eq_normalised("stdout", stdout_str, $expected_out);
        }
    };
}

fn assert_eq_normalised(name: &str, expected: &str, actual: &str) {
    let expected_normalised = normalise_string(expected);
    let actual_normalised = normalise_string(actual);

    assert_eq!(
        expected_normalised, actual_normalised,
        "Expected and actual {} differ:\n\n===== EXPECTED =====\n{}\n\n===== ACTUAL =====\n{}\n\n===== END =====\n",
        name, expected, actual
    );
}

fn normalise_string(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let without_ansi_escapes = String::from_utf8(strip_ansi_escapes::strip(s)).unwrap();

    without_ansi_escapes
        .trim()
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
}

eval_and_assert!(test_factorial, include_str!("factorial.lf"), "3628800", "");

eval_and_assert!(
    test_function_oneliners,
    include_str!("functions.lf"),
    "2 \n 2 \n yes",
    ""
);
