use std::rc::Rc;

use crate::{
    bytecode::Bytecode,
    bytecode_interpreter::RuntimeError,
    runtime_value::{
        function::RuntimeFunction, list::RuntimeList, number::RuntimeNumber, operations::LfAppend,
        set::RuntimeSet,
    },
};

pub mod function;
pub mod list;
pub mod number;
pub mod operations;
pub mod set;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeValue {
    Null,
    Bool(bool),
    Int(isize),
    Num(RuntimeNumber),
    Str(Rc<String>),
    List(RuntimeList),
    Set(RuntimeSet),
    Function(Rc<RuntimeFunction>),
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
            RuntimeValue::Set(_) => "set",
            RuntimeValue::Function(_) => "function",
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a + b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num((*a) + (*b))),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "add", self, other,
            )),
        }
    }

    pub fn sub(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a - b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num((*a) - (*b))),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "subtract", self, other,
            )),
        }
    }

    pub fn mul(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a * b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num((*a) * (*b))),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "multiply", self, other,
            )),
        }
    }

    pub fn eq_bool(&self, other: &Self) -> Result<Self, RuntimeError> {
        Ok(RuntimeValue::Bool(self == other))
    }

    pub fn is_ordering(
        &self,
        other: &Self,
        ordering: std::cmp::Ordering,
    ) -> Result<Self, RuntimeError> {
        self.partial_cmp(other)
            .map(|actual| RuntimeValue::Bool(ordering == actual))
            .ok_or_else(|| RuntimeError::invalid_binary_op_for_types("compare", self, other))
    }

    pub fn less_than(&self, other: &Self) -> Result<Self, RuntimeError> {
        self.is_ordering(other, std::cmp::Ordering::Less)
    }

    pub fn append(&mut self, val: Self) -> Result<(), RuntimeError> {
        match self {
            RuntimeValue::List(list) => list.append(val)?,
            RuntimeValue::Set(set) => set.append(val)?,
            _ => return Err(RuntimeError::NotImplemented(Bytecode::Append)),
        };

        Ok(())
    }

    pub fn address(&self) -> Result<usize, RuntimeError> {
        match self {
            RuntimeValue::Int(i) => Ok(*i as usize),
            _ => Err(RuntimeError::InvalidAddress(self.clone())),
        }
    }

    pub fn bool(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::Null => false,
            RuntimeValue::Int(n) => *n != 0,
            RuntimeValue::Num(n) => n.bool(),
            RuntimeValue::Str(s) => !s.is_empty(),
            RuntimeValue::List(xs) => !xs.as_slice().is_empty(),
            RuntimeValue::Set(xs) => !xs.borrow().is_empty(),
            RuntimeValue::Function(_) => true,
        }
    }
}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn write_items<T: std::fmt::Display>(
            f: &mut std::fmt::Formatter,
            items: impl Iterator<Item = T>,
        ) -> std::fmt::Result {
            let mut first = true;
            for x in items {
                if !first {
                    write!(f, ", ")?;
                }
                first = false;

                write!(f, "{x}")?;
            }
            Ok(())
        }

        match self {
            RuntimeValue::Null => write!(f, "null"),
            RuntimeValue::Bool(b) => write!(f, "{b}"),
            RuntimeValue::Int(n) => write!(f, "{n}"),
            RuntimeValue::Num(n) => write!(f, "{n}"),
            RuntimeValue::Str(s) => write!(f, "{s:?}"),
            RuntimeValue::List(xs) => {
                write!(f, "[")?;
                write_items(f, xs.as_slice().iter())?;
                write!(f, "]")
            }
            RuntimeValue::Set(xs) => {
                write!(f, "{{")?;
                write_items(f, xs.borrow().iter())?;
                write!(f, "}}")
            }
            RuntimeValue::Function(func) => write!(f, "<function@{}>", func.location),
        }
    }
}

impl std::cmp::PartialOrd for RuntimeValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (RuntimeValue::Null, RuntimeValue::Null) => Some(std::cmp::Ordering::Equal),
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a.partial_cmp(b),
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => a.partial_cmp(b),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => a.partial_cmp(b),
            (RuntimeValue::Str(a), RuntimeValue::Str(b)) => a.partial_cmp(b),
            (RuntimeValue::List(a), RuntimeValue::List(b)) => a.partial_cmp(b),
            (RuntimeValue::Set(a), RuntimeValue::Set(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
