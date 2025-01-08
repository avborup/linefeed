use std::io::Read;

pub mod output;

macro_rules! eval_and_assert {
    ($name:ident, $src:expr, $stdout_assertion:expr, $stderr_assertion:expr) => {
        eval_and_assert!($name, $src, "", $stdout_assertion, $stderr_assertion);
    };

    ($name:ident, $src:expr, $stdin_input:expr, $stdout_assertion:expr, $stderr_assertion:expr) => {
        #[test]
        fn $name() -> () {
            let stdin = std::io::Cursor::new($stdin_input);

            let (stdout_str, stderr_str) = crate::helpers::run_program($src, stdin);
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

pub fn run_program(src: &str, mut input: impl Read) -> (String, String) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    linefeed::run_with_handles(src, &mut input, &mut stdout, &mut stderr);
    let stdout_str = std::str::from_utf8(&stdout).unwrap().to_string();
    let stderr_str = std::str::from_utf8(&stderr).unwrap().to_string();

    (stdout_str, stderr_str)
}
