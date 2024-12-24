use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{bytecode_interpreter::RuntimeError, compiler::Instruction};

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Null,
    Bool(bool),
    Int(isize),
    Num(f64),
    Str(Rc<String>),
    List(Rc<RefCell<Vec<RuntimeValue>>>),
    Set(Rc<RefCell<HashSet<RuntimeValue>>>),
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
            RuntimeValue::Set(xs) => {
                xs.borrow_mut().insert(other);
            }
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
                write_items(f, xs.borrow().iter())?;
                write!(f, "]")
            }
            RuntimeValue::Set(xs) => {
                write!(f, "{{")?;
                write_items(f, xs.borrow().iter())?;
                write!(f, "}}")
            }
        }
    }
}

impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeValue::Null, RuntimeValue::Null) => true,
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a == b,
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => a == b,
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => a == b,
            (RuntimeValue::Str(a), RuntimeValue::Str(b)) => a == b,
            (RuntimeValue::List(a), RuntimeValue::List(b)) => a.borrow().eq(&*b.borrow()),
            (RuntimeValue::Set(a), RuntimeValue::Set(b)) => {
                let a = a.borrow();
                let b = b.borrow();

                if a.len() != b.len() {
                    return false;
                }

                a.iter().all(|x| b.contains(x))
            }
            _ => false,
        }
    }
}

impl Eq for RuntimeValue {}

impl std::hash::Hash for RuntimeValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RuntimeValue::Null => 0.hash(state),
            RuntimeValue::Bool(b) => b.hash(state),
            RuntimeValue::Int(i) => i.hash(state),
            RuntimeValue::Num(n) => n.to_bits().hash(state),
            RuntimeValue::Str(s) => s.hash(state),
            RuntimeValue::List(xs) => xs.borrow().hash(state),
            RuntimeValue::Set(xs) => {
                let broow = xs.borrow();
                let mut items = broow.iter().collect::<Vec<_>>();
                items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                items.hash(state);
            }
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
            (RuntimeValue::List(a), RuntimeValue::List(b)) => a.borrow().partial_cmp(&*b.borrow()),
            (RuntimeValue::Set(a), RuntimeValue::Set(b)) => {
                Some(Self::cmp_sets(&a.borrow(), &b.borrow()))
            }
            _ => None,
        }
    }
}

impl RuntimeValue {
    fn cmp_sets(a: &HashSet<RuntimeValue>, b: &HashSet<RuntimeValue>) -> std::cmp::Ordering {
        let mut a = a.iter().collect::<Vec<_>>();
        let mut b = b.iter().collect::<Vec<_>>();

        a.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        b.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
    }
}
