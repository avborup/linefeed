use std::{cell::RefCell, rc::Rc};

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
pub struct RuntimeIterator(Rc<RefCell<dyn Iterator<Item = RuntimeValue>>>);

impl RuntimeIterator {
    pub fn next(&self) -> Option<RuntimeValue> {
        self.0.borrow_mut().next()
    }

    pub fn to_vec(&self) -> Vec<RuntimeValue> {
        let mut out = Vec::new();
        while let Some(value) = self.next() {
            out.push(value);
        }
        out
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

impl From<RuntimeList> for RuntimeIterator {
    fn from(list: RuntimeList) -> Self {
        Self(Rc::new(RefCell::new(ListIterator { list, index: 0 })))
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
        let enumerated = RuntimeValue::Tuple(RuntimeTuple::from_vec(vec![index_val, value]));
        self.index += 1;
        Some(enumerated)
    }
}

impl From<EnumeratedListIterator> for RuntimeIterator {
    fn from(iter: EnumeratedListIterator) -> Self {
        Self(Rc::new(RefCell::new(iter)))
    }
}

impl From<RuntimeRange> for RuntimeIterator {
    fn from(range: RuntimeRange) -> Self {
        Self(Rc::new(RefCell::new(RangeIterator::new(range))))
    }
}

impl From<RuntimeString> for RuntimeIterator {
    fn from(s: RuntimeString) -> Self {
        Self::from(RuntimeList::from_vec(
            s.as_str()
                .chars()
                .map(|c| RuntimeValue::Str(RuntimeString::new(c)))
                .collect(),
        ))
    }
}

pub struct EmptyIterator;

impl Iterator for EmptyIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl From<()> for RuntimeIterator {
    fn from(_: ()) -> Self {
        Self(Rc::new(RefCell::new(EmptyIterator)))
    }
}

impl From<RuntimeMap> for RuntimeIterator {
    fn from(map: RuntimeMap) -> Self {
        Self(Rc::new(RefCell::new(MapIterator::from(map))))
    }
}

impl From<RuntimeCounter> for RuntimeIterator {
    fn from(map: RuntimeCounter) -> Self {
        Self(Rc::new(RefCell::new(MapIterator::from(map))))
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
