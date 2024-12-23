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
    pc: usize,
    bp: usize,
}

impl BytecodeInterpreter<std::io::Stdout, std::io::Stderr> {
    pub fn new(program: Vec<Instruction>) -> Self {
        Self {
            program,
            stack: Vec::new(),
            stdout: std::io::stdout(),
            stderr: std::io::stderr(),
            pc: 0,
            bp: 0,
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
            pc: self.pc,
            bp: self.bp,
        }
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        let res = self.run_inner();

        if let Err(err) = &res {
            // TODO: Attach span based on source code. Add a mapping from instruction to span in
            // compiler.
        }

        res
    }

    fn run_inner(&mut self) -> Result<(), RuntimeError> {
        loop {
            let instr = &self.program[self.pc];
            self.pc += 1;

            self.dbg_print();

            match instr {
                Instruction::Stop => break Ok(()),

                Instruction::PrintValue => {
                    let val = self.pop_stack()?;
                    writeln!(self.stdout, "{val}").unwrap();
                }

                Instruction::ConstantInt(i) => {
                    self.push_stack(RuntimeValue::Int(*i));
                }

                Instruction::GetBasePtr => {
                    self.push_stack(RuntimeValue::Int(self.bp as isize));
                }

                to_implement => {
                    break Err(RuntimeError::NotImplemented(to_implement.clone()));
                }
            }
        }
    }

    pub fn pop_stack(&mut self) -> Result<RuntimeValue, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::StackUnderflow)
    }

    pub fn push_stack(&mut self, value: RuntimeValue) {
        self.stack.push(value);
    }

    pub fn dbg_print(&self) {
        eprintln!("===== Bytecode Interpreter State =====");
        eprintln!("pc: {}", self.pc);
        eprintln!("bp: {}\n", self.bp);
        eprintln!("Program: {:?}", self.program);
        eprintln!("Stack: {:?}", self.stack);
        eprintln!();
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow,
    NotImplemented(Instruction),
}

impl RuntimeError {
    // FIXME: Use spans to provide location in source code.
    pub fn to_chumsky(&self) -> chumsky::error::Simple<String> {
        match self {
            RuntimeError::StackUnderflow => {
                chumsky::error::Simple::custom(Span::default(), "Stack underflow".to_string())
            }
            RuntimeError::NotImplemented(instr) => chumsky::error::Simple::custom(
                Span::default(),
                format!("Instruction not implemented: {instr:?}"),
            ),
        }
    }
}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeValue::Null => write!(f, "null"),
            RuntimeValue::Bool(b) => write!(f, "{b}"),
            RuntimeValue::Int(n) => write!(f, "{n}"),
            RuntimeValue::Num(n) => write!(f, "{n}"),
            RuntimeValue::Str(s) => write!(f, "{s:?}"),
            RuntimeValue::List(xs) => {
                write!(f, "[")?;
                let mut first = true;
                for x in xs.iter() {
                    if !first {
                        write!(f, ", ")?;
                        first = false;
                    }

                    write!(f, "{x}")?;
                }
                write!(f, "]")
            }
        }
    }
}
