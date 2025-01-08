use std::{cmp::Ordering, ops::Deref, rc::Rc};

use crate::{
    bytecode_interpreter::RuntimeError,
    method::Method,
    runtime_value::{
        function::RuntimeFunction, iterator::RuntimeIterator, list::RuntimeList,
        number::RuntimeNumber, operations::LfAppend, range::RuntimeRange, set::RuntimeSet,
        string::RuntimeString,
    },
};

pub mod function;
pub mod iterator;
pub mod list;
pub mod number;
pub mod operations;
pub mod range;
pub mod set;
pub mod string;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeValue {
    Null,
    Uninit,
    Bool(bool),
    Int(isize),
    Num(RuntimeNumber),
    Str(RuntimeString),
    List(RuntimeList),
    Set(RuntimeSet),
    Function(Rc<RuntimeFunction>),
    Range(Box<RuntimeRange>),
    Iterator(Box<RuntimeIterator>),
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
            RuntimeValue::Uninit => "uninitialized",
            RuntimeValue::Bool(_) => "boolean",
            RuntimeValue::Int(_) => "integer",
            RuntimeValue::Num(_) => "number",
            RuntimeValue::Str(_) => "str",
            RuntimeValue::List(_) => "list",
            RuntimeValue::Set(_) => "set",
            RuntimeValue::Function(_) => "function",
            RuntimeValue::Range(_) => "range",
            RuntimeValue::Iterator(_) => "iterator",
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a + b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num((*a) + (*b))),
            (RuntimeValue::Str(a), RuntimeValue::Str(b)) => Ok(RuntimeValue::Str(a.concat(b))),
            (RuntimeValue::Str(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Str(
                a.concat(&RuntimeString::new(b.to_string())),
            )),
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

    pub fn div(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a / b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num((*a) / (*b))),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "divide", self, other,
            )),
        }
    }

    pub fn modulo(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a % b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(a.modulo(b))),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "modulo", self, other,
            )),
        }
    }

    pub fn index(&self, index: &Self) -> Result<Self, RuntimeError> {
        match self {
            RuntimeValue::List(list) => list.index(index),
            _ => Err(RuntimeError::TypeMismatch(format!(
                "Cannot index into '{}'",
                self.kind_str()
            ))),
        }
    }

    pub fn to_iter(&self) -> Result<Self, RuntimeError> {
        let res = match self {
            RuntimeValue::Iterator(iter) => iter.deref().clone(),
            RuntimeValue::Range(range) => RuntimeIterator::from(range.deref().clone()),
            RuntimeValue::List(list) => RuntimeIterator::from(list.clone()),
            _ => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "Cannot iterate over '{}'",
                    self.kind_str()
                )))
            }
        };

        Ok(RuntimeValue::Iterator(Box::new(res)))
    }

    pub fn next(&self) -> Result<Option<Self>, RuntimeError> {
        match self {
            RuntimeValue::Iterator(iterator) => Ok(iterator.next()),
            _ => Err(RuntimeError::TypeMismatch(format!(
                "Cannot call next on '{}'",
                self.kind_str()
            ))),
        }
    }

    pub fn eq_bool(&self, other: &Self) -> Result<Self, RuntimeError> {
        Ok(RuntimeValue::Bool(self == other))
    }

    pub fn not_eq_bool(&self, other: &Self) -> Result<Self, RuntimeError> {
        Ok(RuntimeValue::Bool(self != other))
    }

    pub fn check_ordering(
        &self,
        other: &Self,
        checker: impl FnOnce(Ordering) -> bool,
    ) -> Result<Self, RuntimeError> {
        self.partial_cmp(other)
            .map(|actual| RuntimeValue::Bool(checker(actual)))
            .ok_or_else(|| RuntimeError::invalid_binary_op_for_types("compare", self, other))
    }

    pub fn less_than(&self, other: &Self) -> Result<Self, RuntimeError> {
        self.check_ordering(other, |actual| actual == Ordering::Less)
    }

    pub fn less_than_or_eq(&self, other: &Self) -> Result<Self, RuntimeError> {
        self.check_ordering(other, |actual| {
            actual == Ordering::Less || actual == Ordering::Equal
        })
    }

    pub fn greater_than(&self, other: &Self) -> Result<Self, RuntimeError> {
        self.check_ordering(other, |actual| actual == Ordering::Greater)
    }

    pub fn greater_than_or_eq(&self, other: &Self) -> Result<Self, RuntimeError> {
        self.check_ordering(other, |actual| {
            actual == Ordering::Greater || actual == Ordering::Equal
        })
    }

    pub fn range(&self, other: &Self) -> Result<Self, RuntimeError> {
        let range = match (self, other) {
            (RuntimeValue::Num(start), RuntimeValue::Num(end)) => {
                RuntimeRange::new(*start, Some(*end))
            }
            (RuntimeValue::Num(start), RuntimeValue::Null) => RuntimeRange::new(*start, None),
            _ => {
                return Err(RuntimeError::invalid_binary_op_for_types(
                    "make range from",
                    self,
                    other,
                ))
            }
        };

        Ok(RuntimeValue::Range(Box::new(range)))
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
            RuntimeValue::Uninit => false,
            RuntimeValue::Int(n) => *n != 0,
            RuntimeValue::Num(n) => n.bool(),
            RuntimeValue::Str(s) => !s.is_empty(),
            RuntimeValue::List(xs) => !xs.as_slice().is_empty(),
            RuntimeValue::Set(xs) => !xs.borrow().is_empty(),
            RuntimeValue::Function(_) => true,
            RuntimeValue::Range(_) => true,
            RuntimeValue::Iterator(_) => true,
        }
    }
}

fn write_items<'a, T: std::fmt::Display>(
    f: &mut std::fmt::Formatter,
    items: impl Iterator<Item = T>,
    formatter: impl Fn(&mut std::fmt::Formatter, &T) -> std::fmt::Result,
) -> std::fmt::Result {
    let mut first = true;
    for x in items {
        if !first {
            write!(f, ", ")?;
        }
        first = false;

        formatter(f, &x)?;
    }
    Ok(())
}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeValue::Null => write!(f, "null"),
            RuntimeValue::Uninit => write!(f, "uninitialized"),
            RuntimeValue::Bool(b) => write!(f, "{b}"),
            RuntimeValue::Int(n) => write!(f, "{n}"),
            RuntimeValue::Num(n) => write!(f, "{n}"),
            RuntimeValue::Str(s) => write!(f, "{s}"),
            RuntimeValue::List(xs) => {
                write!(f, "[")?;
                write_items(f, xs.as_slice().iter(), |f, x| x.repr_fmt(f))?;
                write!(f, "]")
            }
            RuntimeValue::Set(xs) => {
                write!(f, "{{")?;
                write_items(f, xs.borrow().iter(), |f, x| x.repr_fmt(f))?;
                write!(f, "}}")
            }
            RuntimeValue::Function(func) => write!(f, "<function@{}>", func.location),
            RuntimeValue::Range(range) => write!(f, "{range}"),
            RuntimeValue::Iterator(iterator) => write!(f, "{iterator}"),
        }
    }
}

impl RuntimeValue {
    /// The "repr" string is the equivalent of the Rust Debug format, but from the POV of the
    /// language user. Much like Python. We don't use the Rust Debug format because we want it for
    /// debugging the compiler, interpreter, etc.
    pub fn repr_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeValue::Str(s) => write!(f, "{:?}", s.as_str()),
            _ => {
                use std::fmt::Display;
                self.fmt(f)
            }
        }
    }

    pub fn repr_string(&self) -> String {
        use std::fmt;

        // A little hack to obtain access to a concrete formatter instance:
        // https://users.rust-lang.org/t/reusing-an-fmt-formatter/8531/4
        pub struct Fmt<F>(pub F)
        where
            F: Fn(&mut fmt::Formatter) -> fmt::Result;

        impl<F> fmt::Debug for Fmt<F>
        where
            F: Fn(&mut fmt::Formatter) -> fmt::Result,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                (self.0)(f)
            }
        }

        format!("{:?}", Fmt(|f| self.repr_fmt(f)))
    }
}

impl std::cmp::PartialOrd for RuntimeValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (RuntimeValue::Null, RuntimeValue::Null) => Some(Ordering::Equal),
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

// Method implementations
impl RuntimeValue {
    pub fn append(&mut self, val: Self) -> Result<(), RuntimeError> {
        match self {
            RuntimeValue::List(list) => list.append(val)?,
            RuntimeValue::Set(set) => set.append(val)?,
            _ => return Err(RuntimeError::invalid_method_for_type(Method::Append, self)),
        };

        Ok(())
    }

    pub fn to_uppercase(&self) -> Result<Self, RuntimeError> {
        let RuntimeValue::Str(s) = self else {
            return Err(RuntimeError::invalid_method_for_type(
                Method::ToUpperCase,
                self,
            ));
        };

        Ok(RuntimeValue::Str(s.to_uppercase()))
    }

    pub fn to_lowercase(&self) -> Result<Self, RuntimeError> {
        let RuntimeValue::Str(s) = self else {
            return Err(RuntimeError::invalid_method_for_type(
                Method::ToLowerCase,
                self,
            ));
        };

        Ok(RuntimeValue::Str(s.to_lowercase()))
    }

    pub fn split(&self, by: &Self) -> Result<Self, RuntimeError> {
        let RuntimeValue::Str(s) = self else {
            return Err(RuntimeError::invalid_method_for_type(Method::Split, self));
        };

        let RuntimeValue::Str(delimiter) = by else {
            return Err(RuntimeError::TypeMismatch(format!(
                "Cannot split string by type '{}'",
                by.kind_str()
            )));
        };

        let list = s.split(delimiter);

        Ok(RuntimeValue::List(list))
    }

    pub fn lines(&self) -> Result<Self, RuntimeError> {
        let RuntimeValue::Str(s) = self else {
            return Err(RuntimeError::invalid_method_for_type(Method::Split, self));
        };

        Ok(RuntimeValue::List(s.lines()))
    }
}
