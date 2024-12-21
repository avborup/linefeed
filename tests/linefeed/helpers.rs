pub mod output;

macro_rules! eval_and_assert {
    ($name:ident, $src:expr, $stdout_assertion:expr, $stderr_assertion:expr) => {
        #[test]
        fn $name() -> () {
            let (stdout_str, stderr_str) = crate::helpers::run_program($src);
            let output = crate::helpers::output::Output {
                stdout: stdout_str,
                stderr: stderr_str,
            };

            output
                .assert(
                    crate::helpers::output::OutputSource::StdErr,
                    $stderr_assertion,
                )
                .assert(
                    crate::helpers::output::OutputSource::StdOut,
                    $stdout_assertion,
                );
        }
    };
}

pub(crate) use eval_and_assert;

pub fn run_program(src: &str) -> (String, String) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let interpreter = linefeed::interpreter::Interpreter::new_with_output(&mut stdout, &mut stderr);

    linefeed::run_with_interpreter(interpreter, src);
    let stdout_str = std::str::from_utf8(&stdout).unwrap().to_string();
    let stderr_str = std::str::from_utf8(&stderr).unwrap().to_string();

    (stdout_str, stderr_str)
}
