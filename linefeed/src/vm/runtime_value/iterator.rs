use std::{cell::RefCell, convert::identity, rc::Rc};

use crate::vm::runtime_value::{
    counter::RuntimeCounter,
    list::RuntimeList,
    map::{MapIterator, RuntimeMap},
    number::RuntimeNumber,
    range::{RangeIterator, RuntimeRange},
    string::RuntimeString,
    tuple::RuntimeTuple,
    RuntimeValue,
};

#[derive(Clone)]
pub struct RuntimeIterator(Rc<RefCell<IteratorKind>>);

enum IteratorKind {
    List(ListIterator),
    Tuple(TupleIterator),
    Range(RangeIterator),
    Map(MapIterator),
    Enumerated(EnumeratedListIterator),
    String(StringIterator),
    Empty,
}

impl RuntimeIterator {
    pub fn next(&self) -> Option<RuntimeValue> {
        match &mut *self.0.borrow_mut() {
            IteratorKind::List(iter) => iter.next(),
            IteratorKind::Tuple(iter) => iter.next(),
            IteratorKind::Range(iter) => iter.next(),
            IteratorKind::Map(iter) => iter.next(),
            IteratorKind::Enumerated(iter) => iter.next(),
            IteratorKind::String(iter) => iter.next(),
            IteratorKind::Empty => None,
        }
    }

    pub fn to_vec(&self) -> Vec<RuntimeValue> {
        self.map_to_vec(identity)
    }

    pub fn len(&self) -> usize {
        match &*self.0.borrow() {
            IteratorKind::List(iter) => iter.list.len().saturating_sub(iter.index),
            IteratorKind::Tuple(iter) => iter.tuple.len().saturating_sub(iter.index),
            IteratorKind::Range(iter) => iter.len().unwrap_or(usize::MAX),
            IteratorKind::Map(iter) => iter.len(),
            IteratorKind::Enumerated(iter) => iter.list.len().saturating_sub(iter.index),
            IteratorKind::String(iter) => iter.chars.len().saturating_sub(iter.index),
            IteratorKind::Empty => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn map_to_vec<F>(&self, f: F) -> Vec<RuntimeValue>
    where
        F: Fn(RuntimeValue) -> RuntimeValue,
    {
        let mut out = Vec::with_capacity(self.len());
        while let Some(value) = self.next() {
            out.push(f(value));
        }
        out
    }

    pub fn fold<T, F>(&self, init: T, f: F) -> T
    where
        F: Fn(T, RuntimeValue) -> T,
    {
        let mut acc = init;
        while let Some(value) = self.next() {
            acc = f(acc, value);
        }
        acc
    }

    pub fn try_fold<T, E, F>(&self, init: T, f: F) -> Result<T, E>
    where
        F: Fn(T, RuntimeValue) -> Result<T, E>,
    {
        let mut acc = init;
        while let Some(value) = self.next() {
            acc = f(acc, value)?;
        }
        Ok(acc)
    }
}

struct ListIterator {
    list: RuntimeList,
    index: usize,
}

impl Iterator for ListIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.list.as_slice().get(self.index).cloned();
        self.index += 1;
        value
    }
}

struct TupleIterator {
    tuple: RuntimeTuple,
    index: usize,
}

impl Iterator for TupleIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.tuple.as_slice().get(self.index).cloned();
        self.index += 1;
        value
    }
}

pub struct EnumeratedListIterator {
    list: RuntimeList,
    index: usize,
}

impl EnumeratedListIterator {
    pub fn new(list: RuntimeList) -> Self {
        Self { list, index: 0 }
    }
}

impl Iterator for EnumeratedListIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.list.as_slice().get(self.index).cloned()?;
        let index_val = RuntimeValue::Num(RuntimeNumber::from(self.index));
        let enumerated = RuntimeValue::from((index_val, value));
        self.index += 1;
        Some(enumerated)
    }
}

pub struct StringIterator {
    chars: Vec<RuntimeString>,
    index: usize,
}

impl StringIterator {
    pub fn new(s: &RuntimeString) -> Self {
        let chars = s.as_str().chars().map(RuntimeString::new).collect();
        Self { chars, index: 0 }
    }
}

impl Iterator for StringIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.chars.get(self.index).cloned()?;
        self.index += 1;
        Some(RuntimeValue::Str(ch))
    }
}

impl From<RuntimeList> for RuntimeIterator {
    fn from(list: RuntimeList) -> Self {
        Self(Rc::new(RefCell::new(IteratorKind::List(ListIterator {
            list,
            index: 0,
        }))))
    }
}

impl From<RuntimeTuple> for RuntimeIterator {
    fn from(tuple: RuntimeTuple) -> Self {
        Self(Rc::new(RefCell::new(IteratorKind::Tuple(TupleIterator {
            tuple,
            index: 0,
        }))))
    }
}

impl From<RuntimeRange> for RuntimeIterator {
    fn from(range: RuntimeRange) -> Self {
        Self(Rc::new(RefCell::new(IteratorKind::Range(
            RangeIterator::new(range),
        ))))
    }
}

impl From<RuntimeString> for RuntimeIterator {
    fn from(s: RuntimeString) -> Self {
        Self(Rc::new(RefCell::new(IteratorKind::String(
            StringIterator::new(&s),
        ))))
    }
}

impl From<EnumeratedListIterator> for RuntimeIterator {
    fn from(iter: EnumeratedListIterator) -> Self {
        Self(Rc::new(RefCell::new(IteratorKind::Enumerated(iter))))
    }
}

impl From<RuntimeMap> for RuntimeIterator {
    fn from(map: RuntimeMap) -> Self {
        Self(Rc::new(RefCell::new(IteratorKind::Map(MapIterator::from(
            map,
        )))))
    }
}

impl From<RuntimeCounter> for RuntimeIterator {
    fn from(counter: RuntimeCounter) -> Self {
        Self(Rc::new(RefCell::new(IteratorKind::Map(MapIterator::from(
            counter,
        )))))
    }
}

impl From<()> for RuntimeIterator {
    fn from(_: ()) -> Self {
        Self(Rc::new(RefCell::new(IteratorKind::Empty)))
    }
}

impl std::cmp::PartialEq for RuntimeIterator {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::cmp::Eq for RuntimeIterator {}

impl std::fmt::Debug for RuntimeIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RuntimeIterator({:p})", self.0.as_ptr())
    }
}

impl std::fmt::Display for RuntimeIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<iterator@{:?}>", self.0.as_ptr())
    }
}

impl std::hash::Hash for RuntimeIterator {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state)
    }
}
