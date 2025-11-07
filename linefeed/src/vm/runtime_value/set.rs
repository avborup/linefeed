use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashSet;

use crate::vm::{
    runtime_value::{iterator::RuntimeIterator, RuntimeValue},
    RuntimeError,
};

#[derive(Debug, Clone)]
pub struct RuntimeSet<'gc>(Rc<RefCell<FxHashSet<RuntimeValue<'gc>>>>);

impl<'gc> RuntimeSet<'gc> {
    pub fn new() -> Self {
        Self::from_set(FxHashSet::default())
    }

    pub fn from_set(set: FxHashSet<RuntimeValue<'gc>>) -> Self {
        Self(Rc::new(RefCell::new(set)))
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, FxHashSet<RuntimeValue<'gc>>> {
        self.0.borrow()
    }

    pub fn append(&self, other: RuntimeValue<'gc>) -> Result<(), RuntimeError> {
        self.0.borrow_mut().insert(other);
        Ok(())
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

    pub fn contains(&self, value: &RuntimeValue<'gc>) -> bool {
        self.0.borrow().contains(value)
    }
}

impl Default for RuntimeSet<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for RuntimeSet<'_> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.borrow();
        let b = other.0.borrow();

        a.len() == b.len() && a.iter().all(|item| b.contains(item))
    }
}

impl<'gc> TryFrom<RuntimeIterator<'gc>> for RuntimeSet<'gc> {
    type Error = RuntimeError;

    fn try_from(iter: RuntimeIterator<'gc>) -> Result<Self, Self::Error> {
        let mut map = FxHashSet::default();
        while let Some(val) = iter.next() {
            map.insert(val);
        }
        Ok(Self::from_set(map))
    }
}

impl Eq for RuntimeSet<'_> {}

impl std::hash::Hash for RuntimeSet<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let set = self.0.borrow();
        let mut items = set.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        items.hash(state);
    }
}

impl std::cmp::PartialOrd for RuntimeSet<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.0.borrow();
        let b = other.0.borrow();
        a.len().partial_cmp(&b.len())
    }
}
