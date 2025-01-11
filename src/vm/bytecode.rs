use std::{collections::HashMap, rc::Rc};

use crate::{
    compiler::{
        ir_value::IrValue, method::Method, stdlib_fn::StdlibFn, CompileError, Instruction, Label,
        Program,
    },
    vm::runtime_value::{
        function::RuntimeFunction, list::RuntimeList, regex::RuntimeRegex, set::RuntimeSet,
        string::RuntimeString, tuple::RuntimeTuple, RuntimeValue,
    },
};

#[derive(Debug, Clone)]
pub enum Bytecode {
    // Variables
    Load,
    Store,

    // Values
    Value(RuntimeValue),
    ConstantInt(isize),

    // Stack manipulation
    Pop,
    RemoveIndex,
    Swap,
    Dup,
    GetStackPtr,
    SetStackPtr,

    // Binary operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Range,
    Xor,

    // Logic
    Not,

    // Control flow
    Stop,
    Goto(usize),
    IfTrue(usize),
    IfFalse(usize),

    // Functions
    GetBasePtr,
    Call(usize),
    Return,

    // Builtins
    PrintValue(usize),
    ReadInput,
    Index,
    NextIter,
    ToIter,
    ParseInt,
    ToList,
    ToTuple,
    Product,
    Sum,
    ReprString,

    // Methods
    Append,
    ToUpperCase,
    ToLowerCase,
    Split,
    SplitLines,
    Join(usize),
    Length,
    Count,
    FindAll,
}

impl Bytecode {
    pub fn from_instruction(
        instruction: Instruction,
        label_mapper: &LabelMapper,
    ) -> Result<Option<Self>, CompileError> {
        let bytecode = match instruction {
            Instruction::Label(_) => return Ok(None),
            Instruction::Load => Bytecode::Load,
            Instruction::Store => Bytecode::Store,
            Instruction::GetBasePtr => Bytecode::GetBasePtr,
            Instruction::Value(value) => {
                Bytecode::Value(Self::into_runtime_value_with_mapper(value, label_mapper)?)
            }
            Instruction::ConstantInt(i) => Bytecode::ConstantInt(i),
            Instruction::Add => Bytecode::Add,
            Instruction::Sub => Bytecode::Sub,
            Instruction::Mul => Bytecode::Mul,
            Instruction::Div => Bytecode::Div,
            Instruction::Mod => Bytecode::Mod,
            Instruction::Eq => Bytecode::Eq,
            Instruction::NotEq => Bytecode::NotEq,
            Instruction::Less => Bytecode::Less,
            Instruction::LessEq => Bytecode::LessEq,
            Instruction::Greater => Bytecode::Greater,
            Instruction::GreaterEq => Bytecode::GreaterEq,
            Instruction::Range => Bytecode::Range,
            Instruction::Xor => Bytecode::Xor,
            Instruction::Not => Bytecode::Not,
            Instruction::Stop => Bytecode::Stop,
            Instruction::Goto(label) => Bytecode::Goto(label_mapper.get(label)?),
            Instruction::IfTrue(label) => Bytecode::IfTrue(label_mapper.get(label)?),
            Instruction::IfFalse(label) => Bytecode::IfFalse(label_mapper.get(label)?),
            Instruction::Pop => Bytecode::Pop,
            Instruction::RemoveIndex => Bytecode::RemoveIndex,
            Instruction::Swap => Bytecode::Swap,
            Instruction::Dup => Bytecode::Dup,
            Instruction::GetStackPtr => Bytecode::GetStackPtr,
            Instruction::SetStackPtr => Bytecode::SetStackPtr,
            Instruction::Call(num_args) => Bytecode::Call(num_args),
            Instruction::Return => Bytecode::Return,
            Instruction::Index => Bytecode::Index,
            Instruction::NextIter => Bytecode::NextIter,
            Instruction::ToIter => Bytecode::ToIter,
            Instruction::StdlibCall(func, num_args) => match func {
                StdlibFn::Print => Bytecode::PrintValue(num_args),
                StdlibFn::Input => Bytecode::ReadInput,
                StdlibFn::ParseInt => Bytecode::ParseInt,
                StdlibFn::ToList => Bytecode::ToList,
                StdlibFn::ToTuple => Bytecode::ToTuple,
                StdlibFn::Repr => Bytecode::ReprString,
                StdlibFn::Product => Bytecode::Product,
                StdlibFn::Sum => Bytecode::Sum,
            },
            Instruction::MethodCall(method, num_args) => match method {
                Method::Append => Bytecode::Append,
                Method::ToUpperCase => Bytecode::ToUpperCase,
                Method::ToLowerCase => Bytecode::ToLowerCase,
                Method::Split => Bytecode::Split,
                Method::SplitLines => Bytecode::SplitLines,
                Method::Join => Bytecode::Join(num_args),
                Method::Length => Bytecode::Length,
                Method::Count => Bytecode::Count,
                Method::FindAll => Bytecode::FindAll,
            },
        };

        Ok(Some(bytecode))
    }

    fn into_runtime_value_with_mapper(
        value: IrValue,
        label_mapper: &LabelMapper,
    ) -> Result<RuntimeValue, CompileError> {
        let res = match value {
            IrValue::Null => RuntimeValue::Null,
            IrValue::Uninit => RuntimeValue::Uninit,
            IrValue::Bool(b) => RuntimeValue::Bool(b),
            IrValue::Int(i) => RuntimeValue::Int(i),
            IrValue::Num(n) => RuntimeValue::Num(n),
            IrValue::Str(s) => RuntimeValue::Str(RuntimeString::new(s)),
            IrValue::List(xs) => {
                let items = xs
                    .into_iter()
                    .map(|item| Self::into_runtime_value_with_mapper(item, label_mapper))
                    .collect::<Result<_, _>>()?;

                RuntimeValue::List(RuntimeList::from_vec(items))
            }
            IrValue::Tuple(xs) => {
                let items = xs
                    .into_iter()
                    .map(|item| Self::into_runtime_value_with_mapper(item, label_mapper))
                    .collect::<Result<_, _>>()?;

                RuntimeValue::Tuple(RuntimeTuple::from_vec(items))
            }
            IrValue::Set(xs) => {
                let items = xs
                    .into_iter()
                    .map(|item| Self::into_runtime_value_with_mapper(item, label_mapper))
                    .collect::<Result<_, _>>()?;

                RuntimeValue::Set(RuntimeSet::from_set(items))
            }
            IrValue::Function(func) => RuntimeValue::Function(Rc::new(RuntimeFunction {
                location: label_mapper.get(func.location)?,
                arity: func.arity,
            })),
            IrValue::Regex(r) => RuntimeValue::Regex(RuntimeRegex::new(r)),
        };

        Ok(res)
    }
}

impl Program<Instruction> {
    pub fn into_bytecode(self) -> Result<Program<Bytecode>, CompileError> {
        let label_mapper = LabelMapper::from(&self);

        let mut bytecode_program = Program::new();
        for (instruction, span) in self.instructions.into_iter().zip(self.source_map) {
            if let Some(bytecode) = Bytecode::from_instruction(instruction, &label_mapper)? {
                bytecode_program.add_instruction(bytecode, span);
            }
        }

        Ok(bytecode_program)
    }
}

pub struct LabelMapper {
    label_locations: HashMap<Label, usize>,
}

impl From<&Program<Instruction>> for LabelMapper {
    fn from(program: &Program<Instruction>) -> Self {
        let mut label_locations = HashMap::new();
        let mut pc = 0;

        for instruction in program.instructions.iter() {
            match instruction {
                Instruction::Label(label) => {
                    label_locations.insert(*label, pc);
                }
                _ => {
                    pc += 1;
                }
            }
        }

        Self { label_locations }
    }
}

impl LabelMapper {
    pub fn get(&self, label: Label) -> Result<usize, CompileError> {
        self.label_locations
            .get(&label)
            .copied()
            .ok_or_else(|| CompileError::Plain(format!("Label '{label:?}' not found")))
    }
}