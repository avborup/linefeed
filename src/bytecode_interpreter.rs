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
            // self.dbg_print();

            let instr = &self.program[self.pc];
            self.pc += 1;

            match instr {
                Instruction::Stop => break Ok(()),

                Instruction::PrintValue => {
                    let val = self.pop_stack()?;
                    writeln!(self.stdout, "{val}").unwrap();
                }

                Instruction::ConstantInt(i) => {
                    self.push_stack(RuntimeValue::Int(*i));
                }

                Instruction::Value(val) => {
                    self.push_stack(val.clone());
                }

                Instruction::GetBasePtr => {
                    self.push_stack(RuntimeValue::Int(self.bp as isize));
                }

                Instruction::Add => {
                    let a = self.pop_stack()?;
                    let b = self.pop_stack()?;
                    self.push_stack(a.add(&b)?);
                }

                Instruction::Mul => {
                    let a = self.pop_stack()?;
                    let b = self.pop_stack()?;
                    self.push_stack(a.mul(&b)?);
                }

                Instruction::Store => {
                    let val = self.pop_stack()?;
                    let addr = self.pop_stack()?.address()?;
                    self.stack[addr] = val;
                }

                Instruction::Load => {
                    let addr = self.pop_stack()?.address()?;
                    self.push_stack(self.stack[addr].clone());
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
        eprintln!("bp: {}", self.bp);
        eprintln!("Instruction: {:?}\n", self.program.get(self.pc));
        eprintln!("Program: {:?}", self.program);
        eprintln!("Stack: {:?}", self.stack);
        eprintln!();
    }
}

impl RuntimeValue {
    pub fn add(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a + b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(a + b)),
            (RuntimeValue::Int(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(*a as f64 + b)),
            (RuntimeValue::Num(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Num(a + *b as f64)),
            _ => Err(RuntimeError::NotImplemented(Instruction::Add)),
        }
    }

    pub fn mul(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a * b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(a * b)),
            (RuntimeValue::Int(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(*a as f64 * b)),
            (RuntimeValue::Num(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Num(a * *b as f64)),
            _ => Err(RuntimeError::NotImplemented(Instruction::Mul)),
        }
    }

    pub fn address(&self) -> Result<usize, RuntimeError> {
        match self {
            RuntimeValue::Int(i) => Ok(*i as usize),
            _ => Err(RuntimeError::InvalidAddress(self.clone())),
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

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow,
    NotImplemented(Instruction),
    InvalidAddress(RuntimeValue),
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
            RuntimeError::InvalidAddress(val) => chumsky::error::Simple::custom(
                Span::default(),
                format!("Invalid address of type {}", val.kind_str()),
            ),
        }
    }
}
