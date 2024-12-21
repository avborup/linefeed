macro_rules! eval_and_assert {
    ($name:ident, $src:expr, $expected_out:expr, $expected_err:expr) => {
        #[test]
        fn $name() -> () {
            let (stdout_str, stderr_str) = run_program($src);
            assert_eq_normalised("stderr", &stderr_str, $expected_err);
            assert_eq_normalised("stdout", &stdout_str, $expected_out);
        }
    };
}

fn run_program(src: &str) -> (String, String) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let interpreter = linefeed::interpreter::Interpreter::new_with_output(&mut stdout, &mut stderr);

    linefeed::run_with_interpreter(interpreter, src);
    let stdout_str = std::str::from_utf8(&stdout).unwrap().to_string();
    let stderr_str = std::str::from_utf8(&stderr).unwrap().to_string();

    (stdout_str, stderr_str)
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

eval_and_assert!(factorial, include_str!("factorial.lf"), "3628800", "");

eval_and_assert!(
    function_oneliners,
    include_str!("functions.lf"),
    "2 \n 2 \n yes",
    ""
);

eval_and_assert!(addition, "print(1 + 2)", "3", "");
eval_and_assert!(subtraction, "print(1 - 2)", "-1", "");
eval_and_assert!(negation, "print(-1 + 3)", "2", "");
eval_and_assert!(multiplication, "print(3 * (1 + 3))", "12", "");
eval_and_assert!(division, "print((2 * 10) / 5)", "4", "");

eval_and_assert!(equals, "print(1 == 1);print(-1 == 1)", "true \n false", "");
eval_and_assert!(
    not_equals,
    "print(1 != 1);print(-1 != 1)",
    "false \n true",
    ""
);
eval_and_assert!(not, "print(!true);print(!false)", "false \n true", "");
