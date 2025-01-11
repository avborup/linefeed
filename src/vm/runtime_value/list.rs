use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::vm::{
    runtime_value::{number::RuntimeNumber, operations::LfAppend, RuntimeValue},
    RuntimeError,
};

#[derive(Debug, Clone)]
pub struct RuntimeList(Rc<RefCell<Vec<RuntimeValue>>>);

impl RuntimeList {
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    pub fn from_vec(vec: Vec<RuntimeValue>) -> Self {
        Self(Rc::new(RefCell::new(vec)))
    }

    pub fn as_slice(&self) -> Ref<'_, [RuntimeValue]> {
        Ref::map(self.0.borrow(), |v| v.as_slice())
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    pub fn deep_clone(&self) -> Self {
        Self::from_vec(self.0.borrow().iter().map(|v| v.deep_clone()).collect())
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue, RuntimeError> {
        let n = index.floor_int();

        let i = if n.is_negative() {
            self.len() as isize - n.abs()
        } else {
            n
        };

        if i < 0 || i as usize >= self.len() {
            return Err(RuntimeError::IndexOutOfBounds(n, self.len()));
        }

        let value = self
            .0
            .borrow()
            .get(i as usize)
            .ok_or_else(|| {
                RuntimeError::InternalBug(format!(
                    "Index {i} is out of bounds for list of length {}",
                    self.len()
                ))
            })?
            .clone();

        Ok(value)
    }
}

impl Default for RuntimeList {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for RuntimeList {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.borrow();
        let b = other.0.borrow();

        a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for RuntimeList {}

impl std::hash::Hash for RuntimeList {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

impl std::cmp::PartialOrd for RuntimeList {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.borrow().partial_cmp(&other.0.borrow())
    }
}

impl LfAppend for RuntimeList {
    fn append(&mut self, other: RuntimeValue) -> Result<(), RuntimeError> {
        self.0.borrow_mut().push(other.clone());
        Ok(())
    }
}
