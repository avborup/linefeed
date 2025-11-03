use std::{
    cell::RefCell,
    io::{Read, Write},
    rc::Rc,
};

pub mod output;

/// A wrapper around Rc<RefCell<Vec<u8>>> that implements Write
struct SharedBuffer(Rc<RefCell<Vec<u8>>>);

impl Write for SharedBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.borrow_mut().flush()
    }
}

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

pub fn run_program(src: &str, input: impl Read + 'static) -> (String, String) {
    let stdout_buf = Rc::new(RefCell::new(Vec::new()));
    let stderr_buf = Rc::new(RefCell::new(Vec::new()));

    let stdout_clone = stdout_buf.clone();
    let stderr_clone = stderr_buf.clone();

    let stdout = SharedBuffer(stdout_buf);
    let stderr = SharedBuffer(stderr_buf);

    linefeed::run_with_handles(src, input, stdout, stderr);

    let stdout_str = std::str::from_utf8(&stdout_clone.borrow()).unwrap().to_string();
    let stderr_str = std::str::from_utf8(&stderr_clone.borrow()).unwrap().to_string();

    (stdout_str, stderr_str)
}
