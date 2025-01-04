use std::{collections::HashMap, rc::Rc};

use crate::{
    compiler::{CompileError, Instruction, Label, Program},
    ir_value::IrValue,
    method::Method,
    runtime_value::{function::RuntimeFunction, list::RuntimeList, set::RuntimeSet, RuntimeValue},
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
    Swap,
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
    PrintValue,
    Index,

    // Methods
    Append,
    ToUpperCase,
    ToLowerCase,
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
            Instruction::Not => Bytecode::Not,
            Instruction::Stop => Bytecode::Stop,
            Instruction::Goto(label) => Bytecode::Goto(label_mapper.get(label)?),
            Instruction::IfTrue(label) => Bytecode::IfTrue(label_mapper.get(label)?),
            Instruction::IfFalse(label) => Bytecode::IfFalse(label_mapper.get(label)?),
            Instruction::Pop => Bytecode::Pop,
            Instruction::Swap => Bytecode::Swap,
            Instruction::GetStackPtr => Bytecode::GetStackPtr,
            Instruction::SetStackPtr => Bytecode::SetStackPtr,
            Instruction::Call(num_args) => Bytecode::Call(num_args),
            Instruction::Return => Bytecode::Return,
            Instruction::PrintValue => Bytecode::PrintValue,
            Instruction::Index => Bytecode::Index,
            Instruction::Method(method) => match method {
                Method::Append => Bytecode::Append,
                Method::ToUpperCase => Bytecode::ToUpperCase,
                Method::ToLowerCase => Bytecode::ToLowerCase,
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
            IrValue::Bool(b) => RuntimeValue::Bool(b),
            IrValue::Int(i) => RuntimeValue::Int(i),
            IrValue::Num(n) => RuntimeValue::Num(n),
            IrValue::Str(s) => RuntimeValue::Str(Rc::new(s)),
            IrValue::List(xs) => {
                let items =
                    xs.0.into_iter()
                        .map(|item| Self::into_runtime_value_with_mapper(item, label_mapper))
                        .collect::<Result<_, _>>()?;

                RuntimeValue::List(RuntimeList::from_vec(items))
            }
            IrValue::Set(xs) => {
                let items =
                    xs.0.into_iter()
                        .map(|item| Self::into_runtime_value_with_mapper(item, label_mapper))
                        .collect::<Result<_, _>>()?;

                RuntimeValue::Set(RuntimeSet::from_set(items))
            }
            IrValue::Function(func) => RuntimeValue::Function(Rc::new(RuntimeFunction {
                location: label_mapper.get(func.location)?,
                arity: func.arity,
            })),
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
