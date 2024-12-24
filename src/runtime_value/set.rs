use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{
    bytecode_interpreter::RuntimeError,
    runtime_value::{operations::LfAppend, RuntimeValue},
};

#[derive(Debug, Clone)]
pub struct RuntimeSet(Rc<RefCell<HashSet<RuntimeValue>>>);

impl RuntimeSet {
    pub fn new() -> Self {
        Self::from_set(HashSet::new())
    }

    pub fn from_set(set: HashSet<RuntimeValue>) -> Self {
        Self(Rc::new(RefCell::new(set)))
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, HashSet<RuntimeValue>> {
        self.0.borrow()
    }
}

impl Default for RuntimeSet {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for RuntimeSet {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.borrow();
        let b = other.0.borrow();

        a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for RuntimeSet {}

impl std::hash::Hash for RuntimeSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let set = self.0.borrow();
        let mut items = set.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        items.hash(state);
    }
}

impl std::cmp::PartialOrd for RuntimeSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.0.borrow();
        let b = other.0.borrow();
        a.len().partial_cmp(&b.len())
    }
}

impl LfAppend for RuntimeSet {
    fn append(&mut self, other: RuntimeValue) -> Result<(), RuntimeError> {
        self.0.borrow_mut().insert(other);
        Ok(())
    }
}
