use std::{cell::RefCell, rc::Rc};

use crate::{bytecode_interpreter::RuntimeError, compiler::Instruction};

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Null,
    Bool(bool),
    Int(isize),
    Num(f64),
    Str(Rc<String>),
    List(Rc<RefCell<Vec<RuntimeValue>>>),
}

const _: () = {
    // Just to make sure that we don't accidentally change the size of RuntimeValue and make
    // cloning super expensive.
    assert!(std::mem::size_of::<RuntimeValue>() == 16);
};

impl RuntimeValue {
    pub fn kind_str(&self) -> &str {
        match self {
            RuntimeValue::Null => "null",
            RuntimeValue::Bool(_) => "boolean",
            RuntimeValue::Int(_) => "integer",
            RuntimeValue::Num(_) => "number",
            RuntimeValue::Str(_) => "str",
            RuntimeValue::List(_) => "list",
        }
    }

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

    pub fn append(&mut self, other: Self) -> Result<(), RuntimeError> {
        match self {
            RuntimeValue::List(xs) => xs.borrow_mut().push(other),
            _ => return Err(RuntimeError::NotImplemented(Instruction::Append)),
        };

        Ok(())
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
                for x in xs.borrow().iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;

                    write!(f, "{x}")?;
                }
                write!(f, "]")
            }
        }
    }
}
