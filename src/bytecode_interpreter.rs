use std::io::Write;

use crate::compiler::{Instruction, RuntimeValue};

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

    pub fn run(&mut self) {
        let mut pc = 0;

        loop {
            let instr = &self.program[pc];
            pc += 1;

            match instr {
                Instruction::Stop => break,

                _ => todo!(),
            }
        }
    }
}
