use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::{
    bytecode_interpreter::RuntimeError,
    runtime_value::{operations::LfAppend, RuntimeValue},
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

    pub fn index(&self, index: &RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
        let RuntimeValue::Num(n) = index else {
            return Err(RuntimeError::TypeMismatch(format!(
                "Expected index to be a number, found {}",
                index.kind_str()
            )));
        };

        let n = n.floor_int();
        let len = self.0.borrow().len();

        if n.unsigned_abs() >= self.0.borrow().len() {
            return Err(RuntimeError::IndexOutOfBounds(n, len));
        }

        let i = if n.is_negative() {
            self.0.borrow().len() - n.unsigned_abs()
        } else {
            n as usize
        };

        let value = self
            .0
            .borrow()
            .get(i)
            .ok_or_else(|| {
                RuntimeError::InternalBug(format!(
                    "Index {i} is out of bounds for list of length {len}",
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
