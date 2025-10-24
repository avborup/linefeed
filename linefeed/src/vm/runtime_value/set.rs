use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::vm::{
    runtime_value::{iterator::RuntimeIterator, operations::LfAppend, RuntimeValue},
    RuntimeError,
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

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    pub fn union(&self, other: &Self) -> Self {
        let mut union = self.0.borrow().clone();
        union.extend(other.0.borrow().iter().cloned());
        Self::from_set(union)
    }

    pub fn intersection(&self, other: &Self) -> Self {
        let intersection = self
            .0
            .borrow()
            .intersection(&other.0.borrow())
            .cloned()
            .collect();

        Self::from_set(intersection)
    }

    pub fn contains(&self, value: &RuntimeValue) -> bool {
        self.0.borrow().contains(value)
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

impl TryFrom<RuntimeIterator> for RuntimeSet {
    type Error = RuntimeError;

    fn try_from(iter: RuntimeIterator) -> Result<Self, Self::Error> {
        let mut map = HashSet::new();
        while let Some(val) = iter.next() {
            map.insert(val);
        }
        Ok(Self::from_set(map))
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
