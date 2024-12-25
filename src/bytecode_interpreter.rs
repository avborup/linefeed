use std::io::Write;

use crate::{ast::Span, bytecode::Bytecode, compiler::Program, runtime_value::RuntimeValue};

pub struct BytecodeInterpreter<O: Write, E: Write> {
    program: Program<Bytecode>,
    stack: Vec<RuntimeValue>,
    pc: usize,
    bp: usize,
    pub stdout: O,
    pub stderr: E,
}

impl BytecodeInterpreter<std::io::Stdout, std::io::Stderr> {
    pub fn new(program: Program<Bytecode>) -> Self {
        Self {
            program,
            stack: vec![RuntimeValue::Null],
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

    pub fn run(&mut self) -> Result<(), (Span, RuntimeError)> {
        self.run_inner().map_err(|err| {
            let source_span = self
                .program
                .source_map
                .get(self.pc)
                .cloned()
                .unwrap_or_default();

            (source_span, err)
        })
    }

    fn run_inner(&mut self) -> Result<(), RuntimeError> {
        loop {
            // self.dbg_print();
            let instr = &self.program.instructions[self.pc];

            match instr {
                Bytecode::Stop => break Ok(()),

                Bytecode::PrintValue => {
                    let val = self.pop_stack()?;
                    writeln!(self.stdout, "{val}").unwrap();
                    self.push_stack(val);
                }

                Bytecode::ConstantInt(i) => {
                    self.push_stack(RuntimeValue::Int(*i));
                }

                Bytecode::Value(val) => {
                    self.push_stack(val.clone());
                }

                Bytecode::Goto(idx) => {
                    self.pc = *idx;
                    continue;
                }

                Bytecode::GetBasePtr => {
                    self.push_stack(RuntimeValue::Int(self.bp as isize));
                }

                Bytecode::Add => {
                    let a = self.pop_stack()?;
                    let b = self.pop_stack()?;
                    self.push_stack(a.add(&b)?);
                }

                Bytecode::Mul => {
                    let a = self.pop_stack()?;
                    let b = self.pop_stack()?;
                    self.push_stack(a.mul(&b)?);
                }

                Bytecode::Store => {
                    let val = self.pop_stack()?;
                    let addr = self.pop_stack()?.address()?;
                    self.stack[addr] = val;
                }

                Bytecode::Append => {
                    let val = self.pop_stack()?;
                    let into = self.peek_stack_mut()?;
                    into.append(val)?;
                }

                Bytecode::Load => {
                    let addr = self.pop_stack()?.address()?;
                    self.push_stack(self.stack[addr].clone());
                }

                Bytecode::Pop => {
                    self.pop_stack()?;
                }

                Bytecode::IfFalse(idx) => {
                    let idx = *idx;
                    let val = self.pop_stack()?;
                    if !val.bool() {
                        self.pc = idx;
                        continue;
                    }
                }

                Bytecode::Not => {
                    let val = self.pop_stack()?;
                    self.push_stack(RuntimeValue::Bool(!val.bool()));
                }

                to_implement => {
                    break Err(RuntimeError::NotImplemented(to_implement.clone()));
                }
            }

            self.pc += 1;
        }
    }

    pub fn pop_stack(&mut self) -> Result<RuntimeValue, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::StackUnderflow)
    }

    pub fn push_stack(&mut self, value: RuntimeValue) {
        self.stack.push(value);
    }

    pub fn peek_stack(&self) -> Result<&RuntimeValue, RuntimeError> {
        self.stack.last().ok_or(RuntimeError::StackUnderflow)
    }

    pub fn peek_stack_mut(&mut self) -> Result<&mut RuntimeValue, RuntimeError> {
        self.stack.last_mut().ok_or(RuntimeError::StackUnderflow)
    }

    pub fn dbg_print(&self) {
        eprintln!("===== Bytecode Interpreter State =====");
        eprintln!("pc: {}", self.pc);
        eprintln!("bp: {}", self.bp);
        eprintln!(
            "Instruction: {:?}\n",
            self.program.instructions.get(self.pc)
        );
        eprintln!("Program: {:?}", self.program.instructions);
        eprintln!("Stack: {:?}", self.stack);
        eprintln!();
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow,
    NotImplemented(Bytecode),
    InvalidAddress(RuntimeValue),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeError::StackUnderflow => write!(f, "Stack underflow"),
            RuntimeError::NotImplemented(instr) => {
                write!(f, "Instruction not implemented: {instr:?}")
            }
            RuntimeError::InvalidAddress(val) => {
                write!(f, "Invalid address of type {}", val.kind_str())
            }
        }
    }
}
