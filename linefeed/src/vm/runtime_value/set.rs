use std::cell::RefCell;

use oxc_allocator::{Allocator, HashSet as AHashSet};

use crate::vm::{runtime_value::RuntimeValue, RuntimeError};

#[derive(Debug)]
pub struct RuntimeSet<'gc>(RefCell<AHashSet<'gc, RuntimeValue<'gc>>>);

impl<'gc> RuntimeSet<'gc> {
    pub fn from_set(set: AHashSet<'gc, RuntimeValue<'gc>>) -> Self {
        Self(RefCell::new(set))
    }

    pub fn from_iter(
        iter: impl IntoIterator<Item = RuntimeValue<'gc>>,
        alloc: &'gc Allocator,
    ) -> Self {
        Self::from_set(AHashSet::from_iter_in(iter, alloc))
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, AHashSet<'gc, RuntimeValue<'gc>>> {
        self.0.borrow()
    }

    pub fn alloc(self, alloc: &'gc Allocator) -> &'gc Self {
        alloc.alloc(self)
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

    pub fn union(&self, other: &Self, alloc: &'gc Allocator) -> &'gc Self {
        let mut union = AHashSet::with_capacity_in(self.len() + other.len(), alloc);
        union.extend(self.borrow().iter().cloned());
        union.extend(other.borrow().iter().cloned());
        Self::from_set(union).alloc(alloc)
    }

    pub fn intersection(&self, other: &Self, alloc: &'gc Allocator) -> &'gc Self {
        let intersection = self.borrow().intersection(&other.borrow()).cloned().fold(
            AHashSet::with_capacity_in(self.len().min(other.len()), alloc),
            |mut acc, item| {
                acc.insert(item);
                acc
            },
        );

        Self::from_set(intersection).alloc(alloc)
    }

    pub fn contains(&self, value: &RuntimeValue<'gc>) -> bool {
        self.0.borrow().contains(value)
    }
}

impl PartialEq for RuntimeSet<'_> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.borrow();
        let b = other.0.borrow();

        a.len() == b.len() && a.iter().all(|item| b.contains(item))
    }
}

// impl<'gc> TryFrom<RuntimeIterator<'gc>> for RuntimeSet<'gc> {
//     type Error = RuntimeError;
//
//     fn try_from(iter: RuntimeIterator<'gc>) -> Result<Self, Self::Error> {
//         let mut map = FxHashSet::default();
//         while let Some(val) = iter.next() {
//             map.insert(val);
//         }
//         Ok(Self::from_set(map))
//     }
// }

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
