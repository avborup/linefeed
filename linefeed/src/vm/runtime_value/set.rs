use std::{cell::RefCell, rc::Rc};

use ouroboros::self_referencing;
use rustc_hash::FxHashSet;

use crate::vm::{
    runtime_value::{iterator::RuntimeIterator, operations::LfAppend, RuntimeValue},
    RuntimeError,
};

#[derive(Debug, Clone)]
pub struct RuntimeSet(Rc<RefCell<FxHashSet<RuntimeValue>>>);

impl RuntimeSet {
    pub fn new() -> Self {
        Self::from_set(FxHashSet::default())
    }

    pub fn from_set(set: FxHashSet<RuntimeValue>) -> Self {
        Self(Rc::new(RefCell::new(set)))
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, FxHashSet<RuntimeValue>> {
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

    pub fn symmetric_difference(&self, other: &Self) -> Self {
        let sym_diff = self
            .0
            .borrow()
            .symmetric_difference(&other.0.borrow())
            .cloned()
            .collect();

        Self::from_set(sym_diff)
    }

    pub fn contains(&self, value: &RuntimeValue) -> bool {
        self.0.borrow().contains(value)
    }

    pub fn remove(&mut self, value: RuntimeValue) {
        self.0.borrow_mut().remove(&value);
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

        a.len() == b.len() && a.iter().all(|item| b.contains(item))
    }
}

impl TryFrom<RuntimeIterator> for RuntimeSet {
    type Error = RuntimeError;

    fn try_from(iter: RuntimeIterator) -> Result<Self, Self::Error> {
        let mut map = FxHashSet::default();
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

#[self_referencing]
struct SetIterCell {
    owner: RuntimeSet,
    #[borrows(owner)]
    #[covariant]
    guard: std::cell::Ref<'this, FxHashSet<RuntimeValue>>,
    #[borrows(guard)]
    #[covariant]
    iter: std::collections::hash_set::Iter<'this, RuntimeValue>,
}

pub struct SetIterator {
    cell: SetIterCell,
    len: usize,
}

impl SetIterator {
    fn new(set: RuntimeSet) -> Self {
        let len = set.borrow().len();
        let cell = SetIterCellBuilder {
            owner: set,
            guard_builder: |owner| owner.borrow(),
            iter_builder: |guard| guard.iter(),
        }
        .build();

        Self { cell, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl From<RuntimeSet> for SetIterator {
    fn from(set: RuntimeSet) -> Self {
        Self::new(set)
    }
}

impl Iterator for SetIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.cell.with_iter_mut(|it| it.next()).cloned()
    }
}
