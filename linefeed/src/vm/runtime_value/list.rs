use std::cell::{Ref, RefCell};

use oxc_allocator::{Allocator, CloneIn, Vec as AVec};
use rustc_hash::FxHashMap;

use crate::vm::{
    runtime_value::{
        number::RuntimeNumber,
        range::RuntimeRange,
        utils::{resolve_index, resolve_slice_indices},
        RuntimeValue,
    },
    RuntimeError,
};

#[derive(Debug)]
pub struct RuntimeList<'gc>(RefCell<AVec<'gc, RuntimeValue<'gc>>>);

impl<'gc> RuntimeList<'gc> {
    pub fn new(alloc: &'gc Allocator) -> Self {
        Self::from_vec(AVec::new_in(alloc))
    }

    pub fn from_iter(
        iter: impl IntoIterator<Item = RuntimeValue<'gc>>,
        alloc: &'gc Allocator,
    ) -> Self {
        Self::from_vec(AVec::from_iter_in(iter, alloc))
    }

    pub fn from_vec(vec: AVec<'gc, RuntimeValue<'gc>>) -> Self {
        Self(RefCell::new(vec))
    }

    pub fn alloc(self, alloc: &'gc Allocator) -> &'gc Self {
        alloc.alloc(self)
    }

    #[inline]
    pub fn borrow(&self) -> Ref<'_, AVec<'gc, RuntimeValue<'gc>>> {
        self.0.borrow()
    }

    pub fn as_slice(&self) -> Ref<'_, [RuntimeValue<'gc>]> {
        Ref::map(self.0.borrow(), |v| v.as_slice())
    }

    pub fn append(&self, other: RuntimeValue<'gc>) -> Result<(), RuntimeError> {
        self.0.borrow_mut().push(other.clone());
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue<'gc>, RuntimeError> {
        let i = resolve_index(self.len(), index)?;

        let value = self
            .0
            .borrow()
            .get(i)
            .ok_or_else(|| {
                RuntimeError::InternalBug(format!(
                    "Index {i} is out of bounds for list of length {}",
                    self.len()
                ))
            })?
            .clone();

        Ok(value)
    }

    pub fn set_index(
        &self,
        index: &RuntimeNumber,
        value: RuntimeValue<'gc>,
    ) -> Result<(), RuntimeError> {
        let i = resolve_index(self.len(), index)?;
        self.0.borrow_mut()[i] = value;
        Ok(())
    }

    pub fn contains(&self, value: &RuntimeValue<'gc>) -> bool {
        self.0.borrow().contains(value)
    }

    pub fn slice(
        &self,
        range: &RuntimeRange,
        alloc: &'gc Allocator,
    ) -> Result<&Self, RuntimeError> {
        let (start, end) = resolve_slice_indices(self.len(), range)?;
        let out = AVec::from_iter_in(self.borrow()[start..end + 1].iter().cloned(), alloc);
        Ok(alloc.alloc(Self::from_vec(out)))
    }

    pub fn sort(&self) {
        self.0
            .borrow_mut()
            .sort_by(|a, b| a.partial_cmp(b).expect("unhandled uncomparable value"));
    }

    pub fn sort_by_key(
        &self,
        mut key_fn: impl FnMut(&RuntimeValue<'gc>) -> Result<RuntimeValue<'gc>, RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let keys = self
            .0
            .borrow()
            .iter()
            .map(|item| {
                let key = key_fn(item)?;
                Ok((item.clone(), key))
            })
            .collect::<Result<FxHashMap<RuntimeValue<'gc>, RuntimeValue<'gc>>, RuntimeError>>()?;

        self.0.borrow_mut().sort_by(|a, b| {
            let key_a = keys.get(a).expect("key not found for item a");
            let key_b = keys.get(b).expect("key not found for item b");
            key_a
                .partial_cmp(key_b)
                .expect("unhandled uncomparable key value")
        });

        Ok(())
    }

    pub fn concat(&self, other: &Self, alloc: &'gc Allocator) -> &Self {
        let mut new_vec = AVec::with_capacity_in(self.len() + other.len(), alloc);
        new_vec.extend_from_slice(&self.as_slice());
        new_vec.extend_from_slice(&other.as_slice());
        alloc.alloc(Self::from_vec(new_vec))
    }
}

impl<'old, 'new> oxc_allocator::CloneIn<'new> for RuntimeList<'old> {
    type Cloned = RuntimeList<'new>;

    fn clone_in(&self, alloc: &'new Allocator) -> Self::Cloned {
        let cloned = self.as_slice().iter().map(|v| v.clone_in(alloc)).fold(
            AVec::with_capacity_in(self.len(), alloc),
            |mut acc, v| {
                acc.push(v);
                acc
            },
        );

        RuntimeList::from_vec(cloned)
    }
}

impl PartialEq for RuntimeList<'_> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.borrow();
        let b = other.0.borrow();

        a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for RuntimeList<'_> {}

impl std::hash::Hash for RuntimeList<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

impl std::cmp::PartialOrd for RuntimeList<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.borrow().partial_cmp(&other.0.borrow())
    }
}
