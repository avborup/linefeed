use std::collections::HashMap;

use crate::{
    compiler::{CompileError, Instruction, Label, Method, Program},
    runtime_value::RuntimeValue,
};

#[derive(Debug, Clone)]
pub enum Bytecode {
    // Variables
    Load,
    Store,

    // Functions
    GetBasePtr,

    // Values
    Value(RuntimeValue),
    ConstantInt(isize),

    // Stack manipulation
    Pop,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,

    // Logic
    Not,

    // Control flow
    Stop,
    Goto(usize),
    IfTrue(usize),
    IfFalse(usize),

    // Methods
    Append,

    // Builtins
    PrintValue,
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
            Instruction::Value(value) => Bytecode::Value(value),
            Instruction::ConstantInt(i) => Bytecode::ConstantInt(i),
            Instruction::Add => Bytecode::Add,
            Instruction::Sub => Bytecode::Sub,
            Instruction::Mul => Bytecode::Mul,
            Instruction::Div => Bytecode::Div,
            Instruction::Not => Bytecode::Not,
            Instruction::Stop => Bytecode::Stop,
            Instruction::Goto(label) => Bytecode::Goto(label_mapper.get(label)?),
            Instruction::IfTrue(label) => Bytecode::IfTrue(label_mapper.get(label)?),
            Instruction::IfFalse(label) => Bytecode::IfFalse(label_mapper.get(label)?),
            Instruction::Pop => Bytecode::Pop,
            Instruction::Method(method) => match method {
                Method::Append => Bytecode::Append,
            },
            Instruction::PrintValue => Bytecode::PrintValue,
        };

        Ok(Some(bytecode))
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
