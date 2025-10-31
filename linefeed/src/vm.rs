use std::{
    collections::HashMap,
    io::{Read, Write},
};

use yansi::Paint;

use crate::{
    compiler::{register_manager::DEFAULT_MAX_REGISTERS, Program},
    grammar::ast::Span,
    vm::{
        bytecode::Bytecode,
        runtime_value::{function::MemoizationKey, string::RuntimeString, RuntimeValue},
    },
};

pub use runtime_error::RuntimeError;

pub mod bytecode;
pub mod runtime_error;
pub mod runtime_value;
pub mod stdlib;

pub struct BytecodeInterpreter<I: Read, O: Write, E: Write> {
    program: Program<Bytecode>,
    // TODO: Optimisation: use stack-allocated array instead of Vec?
    stack: Vec<RuntimeValue>,
    registers: [isize; DEFAULT_MAX_REGISTERS],
    pc: usize,
    bp: usize,
    pub stdin: I,
    pub stdout: O,
    pub stderr: E,
    pub instructions_executed: usize,
    memoized_functions: HashMap<MemoizationKey, RuntimeValue>,
    ongoing_memoizations: HashMap<usize, MemoizationKey>,
}

impl BytecodeInterpreter<std::io::Stdin, std::io::Stdout, std::io::Stderr> {
    pub fn new(program: Program<Bytecode>) -> Self {
        Self {
            program,
            stack: vec![],
            registers: [-1; DEFAULT_MAX_REGISTERS],
            stdin: std::io::stdin(),
            stdout: std::io::stdout(),
            stderr: std::io::stderr(),
            pc: 0,
            bp: 0,
            instructions_executed: 0,
            memoized_functions: HashMap::new(),
            ongoing_memoizations: HashMap::new(),
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

macro_rules! binary_op_swapped {
    ($vm:expr, $op:ident) => {{
        let rhs = $vm.pop_stack()?;
        let lhs = $vm.pop_stack()?;
        $vm.push_stack(rhs.$op(&lhs)?);
    }};
}

macro_rules! unary_mapper_method {
    ($vm:expr, $method:ident) => {{
        let val = $vm.pop_stack()?;
        $vm.push_stack(val.$method()?);
    }};
}

macro_rules! method_with_optional_arg {
    ($vm:expr, $method:ident, $num_args:expr) => {{
        let mut args = $vm.pop_args($num_args)?;
        let arg = args.pop();
        let target = $vm.pop_stack()?;
        $vm.push_stack(target.$method(arg)?);
    }};
}

macro_rules! stdlib_fn {
    ($vm:expr, $fn:ident) => {{
        let val = $vm.pop_stack()?;
        $vm.push_stack(stdlib::$fn(val)?);
    }};

    ($vm:expr, $fn:ident, $num_args:expr) => {{
        let args = $vm.pop_args($num_args)?;
        $vm.push_stack(stdlib::$fn(args)?);
    }};
}

macro_rules! stdlib_fn_with_optional_arg {
    ($vm:expr, $fn:ident, $num_args:expr) => {{
        let mut args = $vm.pop_args($num_args)?;
        let arg = args.pop();
        $vm.push_stack(stdlib::$fn(arg)?);
    }};
}

impl<I, O, E> BytecodeInterpreter<I, O, E>
where
    I: Read,
    O: Write,
    E: Write,
{
    pub fn with_handles<II: Read, OO: Write, EE: Write>(
        self,
        stdin: II,
        stdout: OO,
        stderr: EE,
    ) -> BytecodeInterpreter<II, OO, EE> {
        BytecodeInterpreter {
            program: self.program,
            stack: self.stack,
            registers: self.registers,
            stdin,
            stdout,
            stderr,
            pc: self.pc,
            bp: self.bp,
            instructions_executed: self.instructions_executed,
            memoized_functions: self.memoized_functions,
            ongoing_memoizations: self.ongoing_memoizations,
        }
    }

    pub fn run(&mut self) -> Result<(), (Span, RuntimeError)> {
        self.run_inner().map_err(|err| {
            let source_span = self
                .program
                .source_map
                .get(self.pc - 1)
                .cloned()
                .unwrap_or(Span::new(0, 0));

            (source_span, err)
        })
    }

    fn run_inner(&mut self) -> Result<(), RuntimeError> {
        loop {
            #[cfg(feature = "debug-vm")]
            self.dbg_print();

            let instr = &self.program.instructions[self.pc];
            self.pc += 1;
            self.instructions_executed += 1;

            match instr {
                Bytecode::Stop => break Ok(()),

                Bytecode::ConstantInt(i) => {
                    self.push_stack(RuntimeValue::Int(*i));
                }

                Bytecode::Value(val) => {
                    // Perform a "deep" clone here. Otherwise, the same, shared value is inserted onto the
                    // stack. For things with mutable access, this is BAD. Assign list repeatedly to a
                    // variable? Same list is shared, it's not a new list. Value is no longer referenced on
                    // the stack? Too bad, it's still in the program instructions, so it'll keep living.
                    self.push_stack(val.deep_clone());
                }

                Bytecode::Add => binary_op!(self, add),
                Bytecode::Sub => binary_op!(self, sub),
                Bytecode::Mul => binary_op!(self, mul),
                Bytecode::Div => binary_op!(self, div),
                Bytecode::DivFloor => binary_op!(self, div_floor),
                Bytecode::Mod => binary_op!(self, modulo),
                Bytecode::Pow => binary_op!(self, pow),
                Bytecode::Eq => binary_op!(self, eq_bool),
                Bytecode::NotEq => binary_op!(self, not_eq_bool),
                Bytecode::Less => binary_op!(self, less_than),
                Bytecode::LessEq => binary_op!(self, less_than_or_eq),
                Bytecode::Greater => binary_op!(self, greater_than),
                Bytecode::GreaterEq => binary_op!(self, greater_than_or_eq),
                Bytecode::Range => binary_op!(self, range),
                Bytecode::Xor => binary_op!(self, xor),
                Bytecode::BitwiseAnd => binary_op!(self, bitwise_and),

                Bytecode::Not => {
                    let val = self.pop_stack()?;
                    self.push_stack(RuntimeValue::Bool(!val.bool()));
                }

                Bytecode::Load => {
                    let addr = self.pop_stack()?.address()?;
                    self.push_stack(self.get(addr)?.clone());
                }

                Bytecode::Store => {
                    let addr = self.pop_stack()?.address()?;
                    let val = self.peek_stack()?.clone();
                    self.set(addr, val)?;
                }

                Bytecode::Pop => {
                    self.pop_stack()?;
                }

                Bytecode::RemoveIndex => {
                    let index = self.pop_stack()?.address()?;
                    debug_assert!(index < self.stack.len());
                    self.stack.remove(index);
                }

                Bytecode::Swap => {
                    self.swap();
                }

                Bytecode::Dup => {
                    let val = self.peek_stack()?.clone();
                    self.push_stack(val);
                }

                Bytecode::GetStackPtr => {
                    self.push_stack(RuntimeValue::Int((self.stack.len() - 1) as isize));
                }

                Bytecode::SetStackPtr => {
                    let new_ptr = self.pop_stack()?.address()?;
                    self.stack.truncate(new_ptr + 1);
                }

                Bytecode::SetRegister(reg) => {
                    let reg = *reg;
                    self.registers[reg] = self.pop_stack()?.int()?;
                }

                Bytecode::GetRegister(reg) => {
                    let reg = *reg;
                    self.push_stack(RuntimeValue::Int(self.registers[reg]));
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

                    if func.is_memoized {
                        let args = self.stack[self.stack.len() - num_args..].to_vec();

                        let memo_key = MemoizationKey {
                            func_location,
                            args,
                        };

                        match self.memoized_functions.get(&memo_key) {
                            Some(cached_result) => {
                                self.stack.truncate(func_index);
                                self.push_stack(cached_result.clone());
                                continue;
                            }
                            None => {
                                self.ongoing_memoizations.insert(func_index, memo_key);
                            }
                        }
                    }

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

                    if let Some(memo_key) = self.ongoing_memoizations.remove(&frame_index) {
                        self.memoized_functions.insert(memo_key, return_val.clone());
                    }

                    self.stack.truncate(frame_index);
                    self.push_stack(return_val);
                }

                Bytecode::Append => {
                    let val = self.pop_stack()?;
                    let into = self.peek_stack_mut()?;
                    into.append(val)?;
                }

                Bytecode::Index => {
                    let index = self.pop_stack()?;
                    let into = self.peek_stack_mut()?;
                    let value = into.index(&index)?;
                    *into = value;
                }

                Bytecode::SetIndex => {
                    let value = self.pop_stack()?;
                    let index = self.pop_stack()?;
                    let into = self.peek_stack_mut()?;
                    into.set_index(&index, value)?;
                }

                Bytecode::NextIter => {
                    let iter = self.pop_stack()?;
                    let value = iter.next()?;
                    let has_value = RuntimeValue::Bool(value.is_some());

                    if let Some(value) = value {
                        self.push_stack(value);
                    }
                    self.push_stack(has_value);
                }

                Bytecode::ToIter => unary_mapper_method!(self, to_iter),
                Bytecode::ToUpperCase => unary_mapper_method!(self, to_uppercase),
                Bytecode::ToLowerCase => unary_mapper_method!(self, to_lowercase),
                Bytecode::Split => binary_op!(self, split),
                Bytecode::SplitLines => unary_mapper_method!(self, lines),
                Bytecode::Join(num_args) => method_with_optional_arg!(self, join, *num_args),
                Bytecode::Length => unary_mapper_method!(self, length),
                Bytecode::Count => binary_op!(self, count),
                Bytecode::FindAll => binary_op!(self, find_all),
                Bytecode::Find => binary_op!(self, find),
                Bytecode::IsMatch => binary_op!(self, is_match),
                Bytecode::Contains => binary_op!(self, contains),
                Bytecode::IsIn => binary_op_swapped!(self, contains),
                Bytecode::Sort => unary_mapper_method!(self, sort),
                Bytecode::Enumerate => unary_mapper_method!(self, enumerate),

                Bytecode::ParseInt => stdlib_fn!(self, parse_int),
                Bytecode::ToList => stdlib_fn!(self, to_list),
                Bytecode::ToTuple => stdlib_fn!(self, to_tuple),
                Bytecode::ToMap => stdlib_fn!(self, to_map),
                Bytecode::MapWithDefault => stdlib_fn!(self, map_with_default),
                Bytecode::ToSet(num_args) => stdlib_fn_with_optional_arg!(self, to_set, *num_args),
                Bytecode::ToCounter(num_args) => {
                    stdlib_fn_with_optional_arg!(self, to_counter, *num_args)
                }
                Bytecode::Product => stdlib_fn!(self, mul),
                Bytecode::Sum => stdlib_fn!(self, sum),
                Bytecode::AllTrue(num_args) => stdlib_fn!(self, all, *num_args),
                Bytecode::AnyTrue(num_args) => stdlib_fn!(self, any, *num_args),
                Bytecode::Max(num_args) => stdlib_fn!(self, max, *num_args),
                Bytecode::Min(num_args) => stdlib_fn!(self, min, *num_args),

                Bytecode::PrintValue(num_args) => {
                    let vals = self.pop_args(*num_args)?;

                    let mut last_val = None;
                    for val in vals {
                        if last_val.is_some() {
                            write!(self.stdout, " ").unwrap();
                        }
                        write!(self.stdout, "{val}").unwrap();

                        last_val = Some(val);
                    }
                    writeln!(self.stdout).unwrap();

                    self.push_stack(last_val.unwrap_or(RuntimeValue::Null));
                }

                Bytecode::ReprString => {
                    let val = self.pop_stack()?;
                    let repr = val.repr_string();
                    self.push_stack(RuntimeValue::Str(RuntimeString::new(repr)));
                }

                Bytecode::ReadInput => {
                    let mut input = String::new();
                    self.stdin.read_to_string(&mut input).map_err(|e| {
                        RuntimeError::InternalBug(format!("Failed to read stdin: {e}"))
                    })?;

                    self.push_stack(RuntimeValue::Str(RuntimeString::new(input)));
                }

                Bytecode::RuntimeError(err) => break Err(RuntimeError::Plain(err.clone())),

                #[allow(unreachable_patterns)]
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

    pub fn pop_args(&mut self, num_args: usize) -> Result<Vec<RuntimeValue>, RuntimeError> {
        let mut args = (0..num_args)
            .map(|_| self.pop_stack())
            .collect::<Result<Vec<_>, _>>()?;
        args.reverse();
        Ok(args)
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
        debug_assert!(self.stack.len() >= 2);
        let len = self.stack.len();
        self.stack.swap(len - 1, len - 2);
    }

    pub fn dbg_print(&self) {
        eprintln!("======== Bytecode Interpreter State ========");
        eprintln!("{}", format!("pc: {}", self.pc).dim());
        eprintln!("{}", format!("bp: {}", self.bp).dim());

        fn write_val(val: &RuntimeValue) {
            match val {
                RuntimeValue::Int(_) => eprint!("{}", val.repr_string().yellow()),
                RuntimeValue::Str(_) => eprint!("{}", val.repr_string().green()),
                RuntimeValue::Uninit => eprint!("{}", "uninit".red()),
                // RuntimeValue::List(l) => eprint!("{}", format!("[..; {}]", l.len()).blue()),
                _ => eprint!("{}", format!("{val}").blue()),
            }
        }

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

            write_val(val);
        }
        eprintln!("]\n");

        eprint!("{}: [", "Registers".underline());
        let last_used_register = self
            .registers
            .iter()
            .rposition(|val| *val != -1)
            .unwrap_or(0);
        let mut first = true;
        for val in self.registers.iter().take(last_used_register + 1) {
            if !first {
                eprint!(", ");
            }
            first = false;

            write_val(&RuntimeValue::Int(*val));
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
