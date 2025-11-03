use std::{cmp::Ordering, io::Write, ops::Deref, rc::Rc};

use crate::{
    compiler::method::Method,
    vm::{
        runtime_value::{
            counter::RuntimeCounter,
            function::RuntimeFunction,
            iterator::{EnumeratedListIterator, RuntimeIterator},
            list::RuntimeList,
            map::{MapIterator, RuntimeMap},
            number::RuntimeNumber,
            operations::LfAppend,
            range::RuntimeRange,
            regex::RuntimeRegex,
            set::RuntimeSet,
            string::RuntimeString,
            tuple::RuntimeTuple,
        },
        BytecodeInterpreter, RuntimeError,
    },
};

pub mod counter;
pub mod function;
pub mod iterator;
pub mod list;
pub mod map;
pub mod number;
pub mod operations;
pub mod range;
pub mod regex;
pub mod set;
pub mod string;
pub mod tuple;
mod utils;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeValue {
    Null,
    Uninit,
    Bool(bool),
    Int(isize),
    Num(RuntimeNumber),
    Str(RuntimeString),
    Regex(RuntimeRegex),
    List(RuntimeList),
    Tuple(RuntimeTuple),
    Set(RuntimeSet),
    Map(RuntimeMap),
    Counter(RuntimeCounter),
    Function(Rc<RuntimeFunction>),
    Range(Box<RuntimeRange>),
    Iterator(Box<RuntimeIterator>),
}

const _: () = {
    // Just to make sure that we don't accidentally change the size of RuntimeValue and make
    // cloning super expensive.
    assert!(std::mem::size_of::<RuntimeValue>() == 24);
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
            RuntimeValue::Regex(_) => "regex",
            RuntimeValue::List(_) => "list",
            RuntimeValue::Tuple(_) => "tuple",
            RuntimeValue::Set(_) => "set",
            RuntimeValue::Function(_) => "function",
            RuntimeValue::Range(_) => "range",
            RuntimeValue::Iterator(_) => "iterator",
            RuntimeValue::Map(_) => "map",
            RuntimeValue::Counter(_) => "counter",
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a + b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(a + b)),
            (RuntimeValue::Str(a), RuntimeValue::Str(b)) => Ok(RuntimeValue::Str(a.concat(b))),
            (RuntimeValue::Str(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Str(
                a.concat(&RuntimeString::new(b.to_string())),
            )),
            (RuntimeValue::List(a), RuntimeValue::List(b)) => Ok(RuntimeValue::List(a.concat(b))),
            (RuntimeValue::Set(a), RuntimeValue::Set(b)) => Ok(RuntimeValue::Set(a.union(b))),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "add", self, other,
            )),
        }
    }

    pub fn sub(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a - b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(a - b)),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "subtract", self, other,
            )),
        }
    }

    pub fn mul(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a * b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(a * b)),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "multiply", self, other,
            )),
        }
    }

    pub fn div(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a / b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(a / b)),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "divide", self, other,
            )),
        }
    }

    pub fn div_floor(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a / b)),
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(a.div_floor(b))),
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

    pub fn pow(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Num(a), RuntimeValue::Num(b)) => Ok(RuntimeValue::Num(a.pow(b))),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "power", self, other,
            )),
        }
    }

    pub fn xor(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => Ok(RuntimeValue::Bool(a ^ b)),
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "xor", self, other,
            )),
        }
    }

    pub fn bitwise_and(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeValue::Set(a), RuntimeValue::Set(b)) => {
                Ok(RuntimeValue::Set(a.intersection(b)))
            }
            _ => Err(RuntimeError::invalid_binary_op_for_types(
                "use & on", self, other,
            )),
        }
    }

    pub fn int(&self) -> Result<isize, RuntimeError> {
        let res = match self {
            RuntimeValue::Int(val) => *val,
            _ => {
                return Err(RuntimeError::InternalBug(format!(
                    "Expected integer, found '{}'",
                    self.kind_str()
                )))
            }
        };

        Ok(res)
    }

    pub fn index(&self, index: &Self) -> Result<Self, RuntimeError> {
        let res = match (self, index) {
            (RuntimeValue::List(list), RuntimeValue::Num(i)) => list.index(i)?,
            (RuntimeValue::List(list), RuntimeValue::Range(r)) => {
                RuntimeValue::List(list.slice(r)?)
            }
            (RuntimeValue::Tuple(tuple), RuntimeValue::Num(i)) => tuple.index(i)?,
            (RuntimeValue::Str(s), RuntimeValue::Num(i)) => RuntimeValue::Str(s.index(i)?),
            (RuntimeValue::Str(s), RuntimeValue::Range(r)) => RuntimeValue::Str(s.substr(r)?),
            (RuntimeValue::Map(map), index) => map.get(index),
            (RuntimeValue::Counter(counter), index) => counter.get(index),
            _ => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "Cannot index into '{}' with type '{}'",
                    self.kind_str(),
                    index.kind_str()
                )))
            }
        };

        Ok(res)
    }

    pub fn set_index(&self, index: &Self, value: Self) -> Result<(), RuntimeError> {
        match (self, index) {
            (RuntimeValue::List(list), RuntimeValue::Num(i)) => list.set_index(i, value)?,
            (RuntimeValue::Map(map), index) => map.insert(index.clone(), value),
            _ => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "Cannot index into '{}' with type '{}'",
                    self.kind_str(),
                    index.kind_str()
                )))
            }
        };

        Ok(())
    }

    pub fn to_iter_inner(&self) -> Result<RuntimeIterator, RuntimeError> {
        let iter = match self {
            RuntimeValue::Iterator(iter) => iter.deref().clone(),
            RuntimeValue::Range(range) => RuntimeIterator::from(range.deref().clone()),
            RuntimeValue::List(list) => RuntimeIterator::from(list.clone()),
            RuntimeValue::Str(s) => RuntimeIterator::from(s.clone()),
            RuntimeValue::Map(m) => RuntimeIterator::from(m.clone()),
            _ => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "Cannot iterate over '{}'",
                    self.kind_str()
                )))
            }
        };

        Ok(iter)
    }

    pub fn to_iter(&self) -> Result<Self, RuntimeError> {
        Ok(RuntimeValue::Iterator(Box::new(self.to_iter_inner()?)))
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

    pub fn length(&self) -> Result<Self, RuntimeError> {
        let res = match self {
            RuntimeValue::List(list) => RuntimeValue::Num(RuntimeNumber::from(list.len())),
            RuntimeValue::Str(s) => RuntimeValue::Num(RuntimeNumber::from(s.len())),
            RuntimeValue::Set(s) => RuntimeValue::Num(RuntimeNumber::from(s.len())),
            _ => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "Cannot get length of '{}'",
                    self.kind_str()
                )))
            }
        };

        Ok(res)
    }

    pub fn count(&self, item: &Self) -> Result<Self, RuntimeError> {
        let res = match (self, item) {
            (RuntimeValue::List(list), _) => RuntimeValue::Num(RuntimeNumber::from(
                list.as_slice().iter().filter(|x| *x == item).count(),
            )),
            (RuntimeValue::Str(s), RuntimeValue::Str(sub)) => RuntimeValue::Num(s.count(sub)),
            _ => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "Cannot count '{}' in '{}'",
                    item.kind_str(),
                    self.kind_str()
                )))
            }
        };

        Ok(res)
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

    pub fn sort(
        &self,
        vm: &mut BytecodeInterpreter,
        key_fn: Option<RuntimeValue>,
    ) -> Result<Self, RuntimeError> {
        match self {
            RuntimeValue::List(list) => {
                match key_fn {
                    Some(RuntimeValue::Function(func)) => list.sort_by_key(vm, func.as_ref())?,
                    None => list.sort(),
                    Some(_) => {
                        return Err(RuntimeError::TypeMismatch(
                            "Sort key must be a function".to_string(),
                        ))
                    }
                };

                Ok(RuntimeValue::List(list.clone()))
            }
            _ => Err(RuntimeError::invalid_method_for_type(Method::Sort, self)),
        }
    }

    pub fn enumerate(&self) -> Result<Self, RuntimeError> {
        match self {
            RuntimeValue::List(list) => Ok(RuntimeValue::Iterator(Box::new(
                RuntimeIterator::from(EnumeratedListIterator::new(list.clone())),
            ))),
            _ => Err(RuntimeError::invalid_method_for_type(
                Method::Enumerate,
                self,
            )),
        }
    }

    pub fn range(&self, other: &Self) -> Result<Self, RuntimeError> {
        let range = match (self, other) {
            (RuntimeValue::Num(start), RuntimeValue::Num(end)) => {
                RuntimeRange::new(Some(start.clone()), Some(end.clone()))
            }
            (RuntimeValue::Num(start), RuntimeValue::Null) => {
                RuntimeRange::new(Some(start.clone()), None)
            }
            (RuntimeValue::Null, RuntimeValue::Num(end)) => {
                RuntimeRange::new(None, Some(end.clone()))
            }
            (RuntimeValue::Null, RuntimeValue::Null) => RuntimeRange::new(None, None),
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
            RuntimeValue::Tuple(xs) => !xs.as_slice().is_empty(),
            RuntimeValue::Set(xs) => !xs.borrow().is_empty(),
            RuntimeValue::Map(m) => !m.is_empty(),
            RuntimeValue::Function(_) => true,
            RuntimeValue::Range(_) => true,
            RuntimeValue::Iterator(_) => true,
            RuntimeValue::Regex(_) => true,
            RuntimeValue::Counter(c) => !c.borrow().is_empty(),
        }
    }

    pub fn deep_clone(&self) -> Self {
        match self {
            RuntimeValue::Null => RuntimeValue::Null,
            RuntimeValue::Uninit => RuntimeValue::Uninit,
            RuntimeValue::Bool(b) => RuntimeValue::Bool(*b),
            RuntimeValue::Int(n) => RuntimeValue::Int(*n),
            RuntimeValue::Num(n) => RuntimeValue::Num(n.clone()),
            RuntimeValue::Str(s) => RuntimeValue::Str(s.deep_clone()),
            RuntimeValue::List(xs) => RuntimeValue::List(xs.deep_clone()),
            RuntimeValue::Tuple(xs) => RuntimeValue::Tuple(xs.deep_clone()),
            RuntimeValue::Map(m) => RuntimeValue::Map(m.deep_clone()),
            RuntimeValue::Counter(c) => RuntimeValue::Counter(c.deep_clone()),
            RuntimeValue::Function(_) => self.clone(),
            RuntimeValue::Regex(r) => RuntimeValue::Regex(r.deep_clone()),
            _ => unimplemented!("deep_clone for {:?}", self),
        }
    }
}

fn write_items<T: std::fmt::Display>(
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
            RuntimeValue::Tuple(xs) => {
                write!(f, "(")?;
                write_items(f, xs.as_slice().iter(), |f, x| x.repr_fmt(f))?;
                write!(f, ")")
            }
            RuntimeValue::Set(xs) => {
                write!(f, "{{")?;
                let xs = xs.borrow();
                let mut items = xs.iter().collect::<Vec<_>>();
                items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
                write_items(f, items.iter(), |f, x| x.repr_fmt(f))?;
                write!(f, "}}")
            }
            RuntimeValue::Map(m) => {
                let mut kv_pairs = MapIterator::from(m.clone()).collect::<Vec<_>>();
                kv_pairs
                    .sort_by(|a, b| dbg!(dbg!(a).partial_cmp(dbg!(b))).unwrap_or(Ordering::Equal));

                write!(f, "{{")?;
                write_items(f, kv_pairs.iter(), |f, kv| {
                    let write_item = |f: &mut std::fmt::Formatter, idx: isize| {
                        kv.index(&RuntimeValue::Num(RuntimeNumber::from(idx)))
                            .unwrap()
                            .repr_fmt(f)
                    };

                    write_item(f, 0)?;
                    write!(f, ": ")?;
                    write_item(f, 1)
                })?;
                write!(f, "}}")
            }
            RuntimeValue::Counter(c) => {
                std::fmt::Display::fmt(&RuntimeValue::Map(c.into_runtime_map()), f)
            }
            RuntimeValue::Function(func) => write!(f, "<function@{}>", func.location),
            RuntimeValue::Range(range) => write!(f, "{range}"),
            RuntimeValue::Iterator(iterator) => write!(f, "{iterator}"),
            RuntimeValue::Regex(regex) => write!(f, "{regex}"),
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
            (RuntimeValue::Tuple(a), RuntimeValue::Tuple(b)) => a.partial_cmp(b),
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
            RuntimeValue::Counter(counter) => counter.add(val, 1),
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

    pub fn join(&self, separator: Option<RuntimeValue>) -> Result<Self, RuntimeError> {
        let Ok(Self::Iterator(iter)) = self.to_iter() else {
            return Err(RuntimeError::invalid_method_for_type(Method::Join, self));
        };

        let separator = match separator {
            Some(RuntimeValue::Str(s)) => s,
            None => RuntimeString::new(String::new()),
            Some(val) => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "Cannot join by type '{}'",
                    val.kind_str()
                )))
            }
        };
        let separator = separator.as_str();

        let mut output = Vec::new();
        let mut first = true;
        while let Some(val) = iter.next() {
            if !first {
                write!(&mut output, "{separator}")
                    .map_err(|e| RuntimeError::InternalBug(e.to_string()))?;
            }
            first = false;

            write!(&mut output, "{val}").map_err(|e| RuntimeError::InternalBug(e.to_string()))?;
        }

        let s = String::from_utf8(output).map_err(|e| RuntimeError::InternalBug(e.to_string()))?;

        Ok(RuntimeValue::Str(RuntimeString::new(s)))
    }

    pub fn find_all(&self, search: &Self) -> Result<Self, RuntimeError> {
        match (self, search) {
            (RuntimeValue::Str(input), RuntimeValue::Regex(regex)) => {
                let matches = regex.find_matches(input);
                Ok(RuntimeValue::List(matches))
            }
            _ => Err(RuntimeError::invalid_method_for_type(Method::FindAll, self)),
        }
    }

    pub fn find(&self, search: &Self) -> Result<Self, RuntimeError> {
        match (self, search) {
            (RuntimeValue::Str(input), RuntimeValue::Regex(regex)) => Ok(regex.find_match(input)),
            _ => Err(RuntimeError::invalid_method_for_type(Method::Find, self)),
        }
    }

    pub fn is_match(&self, search: &Self) -> Result<Self, RuntimeError> {
        match (self, search) {
            (RuntimeValue::Str(input), RuntimeValue::Regex(regex)) => {
                Ok(RuntimeValue::Bool(regex.is_match(input)))
            }
            _ => Err(RuntimeError::invalid_method_for_type(Method::IsMatch, self)),
        }
    }

    pub fn contains(&self, item: &Self) -> Result<Self, RuntimeError> {
        let contains = match (self, item) {
            (RuntimeValue::Map(m), k) => m.contains_key(k),
            (RuntimeValue::List(l), v) => l.contains(v),
            (RuntimeValue::Set(l), v) => l.contains(v),
            (RuntimeValue::Tuple(t), v) => t.contains(v),
            (RuntimeValue::Range(r), RuntimeValue::Num(n)) => r.contains(n),
            (RuntimeValue::Str(s1), RuntimeValue::Str(s2)) => s1.contains(s2),
            _ => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "Cannot check if '{}' contains '{}'",
                    self.kind_str(),
                    item.kind_str(),
                )))
            }
        };

        Ok(RuntimeValue::Bool(contains))
    }
}
