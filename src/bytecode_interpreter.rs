use std::io::Write;

use yansi::Paint;

use crate::{ast::Span, bytecode::Bytecode, compiler::Program, runtime_value::RuntimeValue};

pub struct BytecodeInterpreter<O: Write, E: Write> {
    program: Program<Bytecode>,
    // TODO: Optimisation: use stack-allocated array instead of Vec?
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
            stack: vec![],
            stdout: std::io::stdout(),
            stderr: std::io::stderr(),
            pc: 0,
            bp: 0,
        }
    }
}

macro_rules! binary_op {
    ($vm:expr, $op:ident) => {{
        let rhs = $vm.pop_stack()?;
        let lhs = $vm.pop_stack()?;
        $vm.push_stack(lhs.$op(&rhs)?);
    }};
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
            self.dbg_print();
            let instr = &self.program.instructions[self.pc];
            self.pc += 1;

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

                Bytecode::Add => binary_op!(self, add),
                Bytecode::Sub => binary_op!(self, sub),
                Bytecode::Mul => binary_op!(self, mul),
                Bytecode::Eq => binary_op!(self, eq_bool),
                Bytecode::Less => binary_op!(self, less_than),
                Bytecode::LessEq => binary_op!(self, less_than_or_eq),
                Bytecode::Greater => binary_op!(self, greater_than),
                Bytecode::GreaterEq => binary_op!(self, greater_than_or_eq),

                Bytecode::Not => {
                    let val = self.pop_stack()?;
                    self.push_stack(RuntimeValue::Bool(!val.bool()));
                }

                Bytecode::Load => {
                    let addr = self.pop_stack()?.address()?;
                    self.push_stack(self.get(addr)?.clone());
                }

                Bytecode::Store => {
                    self.swap();
                    let addr = self.pop_stack()?.address()?;
                    let val = self.peek_stack()?.clone();
                    self.set(addr, val)?;
                }

                Bytecode::Pop => {
                    self.pop_stack()?;
                }

                Bytecode::IfFalse(idx) => {
                    let idx = *idx;
                    let val = self.pop_stack()?;
                    if !val.bool() {
                        self.pc = idx;
                    }
                }

                Bytecode::IfTrue(idx) => {
                    let idx = *idx;
                    let val = self.pop_stack()?;
                    if val.bool() {
                        self.pc = idx;
                    }
                }

                Bytecode::Goto(idx) => {
                    self.pc = *idx;
                }

                Bytecode::GetBasePtr => {
                    self.push_stack(RuntimeValue::Int(self.bp as isize));
                }

                Bytecode::Call(num_args) => {
                    let num_args = *num_args;

                    let func_index = self.stack.len() - 1 - num_args;
                    let func = match &self.stack[func_index] {
                        RuntimeValue::Function(func) => func,
                        val => {
                            break Err(RuntimeError::TypeMismatch(format!(
                                "Cannot call type {} as a function",
                                val.kind_str()
                            )));
                        }
                    };

                    if func.arity != num_args {
                        break Err(RuntimeError::TypeMismatch(format!(
                            "Expected {} arguments, got {}",
                            func.arity, num_args
                        )));
                    }

                    let func_location = func.location;

                    // Store pc and bp (2 slots), then start new stack frame after that
                    let new_bp = func_index + 2;

                    // First slot is the return address; pop function instance and insert return address
                    self.stack[new_bp - 2] = RuntimeValue::Int(self.pc as isize);
                    // Second slot is the old base pointer
                    self.stack
                        .insert(new_bp - 1, RuntimeValue::Int(self.bp as isize));

                    // And then set the new base pointer and jump to the function
                    self.bp = new_bp;
                    self.pc = func_location;
                }

                Bytecode::Return => {
                    let return_val = self.pop_stack()?;
                    let frame_index = self.bp - 2;

                    let return_addr = self.stack[self.bp - 2].address()?;
                    self.bp = self.stack[self.bp - 1].address()?;
                    self.pc = return_addr;

                    self.stack.truncate(frame_index);
                    self.push_stack(return_val);
                }

                Bytecode::Append => {
                    let val = self.pop_stack()?;
                    let into = self.peek_stack_mut()?;
                    into.append(val)?;
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

    pub fn peek_stack(&self) -> Result<&RuntimeValue, RuntimeError> {
        self.stack.last().ok_or(RuntimeError::StackUnderflow)
    }

    // TODO: It's probably very slow to check this every time, but it provides good diagnostics.
    // Provide feature flag to enable checks?
    pub fn set(&mut self, index: usize, value: RuntimeValue) -> Result<(), RuntimeError> {
        if index >= self.stack.len() {
            return Err(RuntimeError::InternalBug(format!(
                "Tried to set stack index {} but stack length is {}",
                index,
                self.stack.len()
            )));
        }

        self.stack[index] = value;
        Ok(())
    }

    pub fn get(&self, index: usize) -> Result<&RuntimeValue, RuntimeError> {
        if index >= self.stack.len() {
            return Err(RuntimeError::InternalBug(format!(
                "Tried to get stack index {} but stack length is {}",
                index,
                self.stack.len()
            )));
        }

        Ok(&self.stack[index])
    }

    pub fn peek_stack_mut(&mut self) -> Result<&mut RuntimeValue, RuntimeError> {
        self.stack.last_mut().ok_or(RuntimeError::StackUnderflow)
    }

    pub fn swap(&mut self) {
        let len = self.stack.len();
        self.stack.swap(len - 1, len - 2);
    }

    pub fn dbg_print(&self) {
        eprintln!("======== Bytecode Interpreter State ========");
        eprintln!("{}", format!("pc: {}", self.pc).dim());
        eprintln!("{}", format!("bp: {}", self.bp).dim());

        eprint!("{}: [", "Stack".underline());
        let mut first = true;
        for (i, val) in self.stack.iter().enumerate() {
            if !first {
                eprint!(", ");
            }
            first = false;

            if i == self.bp {
                eprint!("{} ", "(bp)".yellow());
            }

            eprint!("{}", format!("{val}").blue());
        }
        eprintln!("]\n");

        eprintln!("{}:", "Instructions".underline());
        for i in (self.pc.saturating_sub(2))..=(self.pc + 2) {
            if i >= self.program.instructions.len() {
                continue;
            }

            let content = format!("{:>3}: {:?}", i, self.program.instructions[i]);
            if i == self.pc {
                eprintln!("-> {}", content.bold());
            } else {
                eprintln!("   {}", content.dim());
            }
        }
        eprintln!();
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow,
    NotImplemented(Bytecode),
    InvalidAddress(RuntimeValue),
    TypeMismatch(String),
    InternalBug(String),
}

impl RuntimeError {
    pub fn invalid_binary_op_for_types(
        action: &str,
        lhs: &RuntimeValue,
        rhs: &RuntimeValue,
    ) -> Self {
        RuntimeError::TypeMismatch(format!(
            "Cannot {action} types '{}' and '{}'",
            lhs.kind_str(),
            rhs.kind_str()
        ))
    }
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
            RuntimeError::TypeMismatch(msg) => {
                write!(f, "Type mismatch: {msg}")
            }
            RuntimeError::InternalBug(msg) => {
                write!(f, "Internal bug: {msg}")
            }
        }
    }
}
