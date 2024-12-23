use std::io::Write;

use crate::{
    ast::Span,
    compiler::{Instruction, RuntimeValue},
};

pub struct BytecodeInterpreter<O: Write, E: Write> {
    program: Vec<Instruction>,
    stack: Vec<RuntimeValue>,
    stdout: O,
    stderr: E,
}

impl BytecodeInterpreter<std::io::Stdout, std::io::Stderr> {
    pub fn new(program: Vec<Instruction>) -> Self {
        Self {
            program,
            stack: Vec::new(),
            stdout: std::io::stdout(),
            stderr: std::io::stderr(),
        }
    }
}

impl<O, E> BytecodeInterpreter<O, E>
where
    O: Write,
    E: Write,
{
    pub fn with_output<OO: Write, EE: Write>(
        self,
        stdout: OO,
        stderr: EE,
    ) -> BytecodeInterpreter<OO, EE> {
        BytecodeInterpreter {
            program: self.program,
            stack: self.stack,
            stdout,
            stderr,
        }
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        let mut pc = 0;

        loop {
            let instr = &self.program[pc];
            pc += 1;

            match instr {
                Instruction::Stop => break Ok(()),

                Instruction::PrintValue => {
                    let val = self.pop_stack()?;
                    writeln!(self.stdout, "{val}").unwrap();
                }

                to_implement => {
                    dbg!(to_implement);
                    todo!()
                }
            }
        }
    }

    pub fn pop_stack(&mut self) -> Result<RuntimeValue, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::StackUnderflow)
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow,
}

impl RuntimeError {
    // FIXME: Use spans to provide location in source code.
    pub fn to_chumsky(&self) -> chumsky::error::Simple<String> {
        match self {
            RuntimeError::StackUnderflow => {
                chumsky::error::Simple::custom(Span::default(), "Stack underflow".to_string())
            }
        }
    }
}
