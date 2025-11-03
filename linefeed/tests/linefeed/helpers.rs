use std::{
    cell::RefCell,
    io::{Read, Write},
    rc::Rc,
};

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

pub fn run_program(src: &str, input: impl Read + 'static) -> (String, String) {
    let stdout = SharedBuffer::new();
    let stderr = SharedBuffer::new();

    linefeed::run_with_handles(src, input, stdout.clone(), stderr.clone());

    let stdout_str = std::str::from_utf8(&stdout.0.borrow()).unwrap().to_string();
    let stderr_str = std::str::from_utf8(&stderr.0.borrow()).unwrap().to_string();

    (stdout_str, stderr_str)
}

/// A wrapper around Rc<RefCell<Vec<u8>>> that implements Write, so we can use the buffer after it
/// has been passed to the VM (as an owned value).
#[derive(Clone)]
struct SharedBuffer(Rc<RefCell<Vec<u8>>>);

impl SharedBuffer {
    fn new() -> Self {
        SharedBuffer(Rc::new(RefCell::new(Vec::new())))
    }
}

impl Write for SharedBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.borrow_mut().flush()
    }
}
