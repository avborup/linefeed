use std::{collections::HashMap, rc::Rc};

use crate::{
    compiler::{
        ir_value::IrValue, method::Method, stdlib_fn::StdlibFn, CompileError, Instruction, Label,
        Program,
    },
    vm::runtime_value::{
        function::RuntimeFunction, list::RuntimeList, map::RuntimeMap, regex::RuntimeRegex,
        set::RuntimeSet, string::RuntimeString, tuple::RuntimeTuple, RuntimeValue,
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

    // Register manipulation
    SetRegister(usize),
    GetRegister(usize),

    // Binary operations
    Add,
    Sub,
    Mul,
    Div,
    DivFloor,
    Mod,
    Pow,
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Range,
    Xor,
    BitwiseAnd,

    // Logic
    Not,

    // Control flow
    Stop,
    Goto(usize),
    IfTrue(usize),
    IfFalse(usize),
    RuntimeError(String),

    // Functions
    GetBasePtr,
    Call(usize),
    Return,

    // Builtins
    PrintValue(usize),
    ReadInput,
    Index,
    SetIndex,
    NextIter,
    ToIter,
    ParseInt,
    ToList,
    ToTuple,
    ToMap,
    MapWithDefault,
    ToSet(usize),
    Product,
    Sum,
    ReprString,
    IsIn,
    AllTrue(usize),
    AnyTrue(usize),
    Max(usize),
    Min(usize),

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
    Find,
    IsMatch,
    Contains,
    Sort,
    Enumerate,
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
            Instruction::DivFloor => Bytecode::DivFloor,
            Instruction::Mod => Bytecode::Mod,
            Instruction::Pow => Bytecode::Pow,
            Instruction::Eq => Bytecode::Eq,
            Instruction::NotEq => Bytecode::NotEq,
            Instruction::Less => Bytecode::Less,
            Instruction::LessEq => Bytecode::LessEq,
            Instruction::Greater => Bytecode::Greater,
            Instruction::GreaterEq => Bytecode::GreaterEq,
            Instruction::Range => Bytecode::Range,
            Instruction::Xor => Bytecode::Xor,
            Instruction::BitwiseAnd => Bytecode::BitwiseAnd,
            Instruction::Not => Bytecode::Not,
            Instruction::Stop => Bytecode::Stop,
            Instruction::Goto(label) => Bytecode::Goto(label_mapper.get(label)?),
            Instruction::IfTrue(label) => Bytecode::IfTrue(label_mapper.get(label)?),
            Instruction::IfFalse(label) => Bytecode::IfFalse(label_mapper.get(label)?),
            Instruction::RuntimeError(msg) => Bytecode::RuntimeError(msg),
            Instruction::Pop => Bytecode::Pop,
            Instruction::RemoveIndex => Bytecode::RemoveIndex,
            Instruction::Swap => Bytecode::Swap,
            Instruction::Dup => Bytecode::Dup,
            Instruction::GetStackPtr => Bytecode::GetStackPtr,
            Instruction::SetStackPtr => Bytecode::SetStackPtr,
            Instruction::SetRegister(register) => Bytecode::SetRegister(register),
            Instruction::GetRegister(register) => Bytecode::GetRegister(register),
            Instruction::Call(num_args) => Bytecode::Call(num_args),
            Instruction::Return => Bytecode::Return,
            Instruction::Index => Bytecode::Index,
            Instruction::SetIndex => Bytecode::SetIndex,
            Instruction::NextIter => Bytecode::NextIter,
            Instruction::ToIter => Bytecode::ToIter,
            Instruction::IsIn => Bytecode::IsIn,
            Instruction::StdlibCall(func, num_args) => match func {
                StdlibFn::Print => Bytecode::PrintValue(num_args),
                StdlibFn::Input => Bytecode::ReadInput,
                StdlibFn::ParseInt => Bytecode::ParseInt,
                StdlibFn::ToList => Bytecode::ToList,
                StdlibFn::ToTuple => Bytecode::ToTuple,
                StdlibFn::ToMap => Bytecode::ToMap,
                StdlibFn::MapWithDefault => Bytecode::MapWithDefault,
                StdlibFn::ToSet => Bytecode::ToSet(num_args),
                StdlibFn::Repr => Bytecode::ReprString,
                StdlibFn::Product => Bytecode::Product,
                StdlibFn::Sum => Bytecode::Sum,
                StdlibFn::All => Bytecode::AllTrue(num_args),
                StdlibFn::Any => Bytecode::AnyTrue(num_args),
                StdlibFn::Max => Bytecode::Max(num_args),
                StdlibFn::Min => Bytecode::Min(num_args),
            },
            Instruction::MethodCall(method, num_args) => match method {
                Method::Append | Method::Add => Bytecode::Append,
                Method::ToUpperCase => Bytecode::ToUpperCase,
                Method::ToLowerCase => Bytecode::ToLowerCase,
                Method::Split => Bytecode::Split,
                Method::SplitLines => Bytecode::SplitLines,
                Method::Join => Bytecode::Join(num_args),
                Method::Length => Bytecode::Length,
                Method::Count => Bytecode::Count,
                Method::FindAll => Bytecode::FindAll,
                Method::Find => Bytecode::Find,
                Method::IsMatch => Bytecode::IsMatch,
                Method::Contains => Bytecode::Contains,
                Method::Sort => Bytecode::Sort,
                Method::Enumerate => Bytecode::Enumerate,
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
            IrValue::Map(m) => {
                let map = m
                    .into_iter()
                    .map(|(key, value)| {
                        Ok((
                            Self::into_runtime_value_with_mapper(key, label_mapper)?,
                            Self::into_runtime_value_with_mapper(value, label_mapper)?,
                        ))
                    })
                    .collect::<Result<_, _>>()?;

                RuntimeValue::Map(RuntimeMap::from_map(map))
            }
            IrValue::Function(func) => RuntimeValue::Function(Rc::new(RuntimeFunction {
                location: label_mapper.get(func.location)?,
                arity: func.arity,
            })),
            IrValue::Regex(s, modifiers) => {
                let regex = RuntimeRegex::compile(&s, modifiers)
                    .map_err(|e| CompileError::Plain(format!("Invalid regex: {e}")))?;

                RuntimeValue::Regex(regex)
            }
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
