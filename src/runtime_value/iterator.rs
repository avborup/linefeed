use std::{cell::RefCell, rc::Rc};

use crate::runtime_value::{
    list::RuntimeList,
    range::{RangeIterator, RuntimeRange},
    RuntimeValue,
};

#[derive(Clone)]
pub struct RuntimeIterator(Rc<RefCell<dyn Iterator<Item = RuntimeValue>>>);

impl RuntimeIterator {
    pub fn next(&self) -> Option<RuntimeValue> {
        self.0.borrow_mut().next()
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

impl From<RuntimeRange> for RuntimeIterator {
    fn from(range: RuntimeRange) -> Self {
        Self(Rc::new(RefCell::new(RangeIterator::new(range))))
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