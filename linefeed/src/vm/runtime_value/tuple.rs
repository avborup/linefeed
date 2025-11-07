use std::cell::{Ref, RefCell};

use oxc_allocator::Allocator;

use crate::vm::{
    runtime_value::{number::RuntimeNumber, utils::resolve_index, RuntimeValue},
    RuntimeError,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// TODO: just use cell
pub struct RuntimeTuple<'gc>(RefCell<Vec<RuntimeValue<'gc>>>);

impl<'gc> RuntimeTuple<'gc> {
    pub fn from_vec(vec: Vec<RuntimeValue<'gc>>) -> Self {
        Self(RefCell::new(vec))
    }

    pub fn as_slice(&self) -> Ref<'_, [RuntimeValue<'gc>]> {
        Ref::map(self.0.borrow(), |v| v.as_slice())
    }

    pub fn borrow(&self) -> Ref<'_, Vec<RuntimeValue<'gc>>> {
        self.0.borrow()
    }

    pub fn len(&self) -> usize {
        self.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue<'gc>, RuntimeError> {
        let i = resolve_index(self.len(), index)?;

        self.borrow()
            .get(i)
            .cloned()
            .ok_or_else(|| RuntimeError::IndexOutOfBounds(i as isize, self.len()))
    }

    pub fn contains(&self, value: &RuntimeValue<'gc>) -> bool {
        self.borrow().iter().any(|v| v == value)
    }

    pub fn element_wise_add(
        &self,
        other: &Self,
        alloc: &'gc Allocator,
    ) -> Result<&Self, RuntimeError> {
        if self.len() != other.len() {
            return Err(RuntimeError::TypeMismatch(format!(
                "Cannot add tuples of different lengths: {} and {}",
                self.len(),
                other.len()
            )));
        }

        let result: Result<Vec<RuntimeValue<'gc>>, RuntimeError> = self
            .borrow()
            .iter()
            .zip(other.borrow().iter())
            .map(|(a, b)| a.add(b, alloc))
            .collect();

        let tup = alloc.alloc(RuntimeTuple::from_vec(result?));

        Ok(tup)
    }

    pub fn scalar_multiply(
        &self,
        scalar: &RuntimeValue<'gc>,
        alloc: &'gc Allocator,
    ) -> Result<&Self, RuntimeError> {
        let result: Result<Vec<RuntimeValue<'gc>>, RuntimeError> = self
            .borrow()
            .iter()
            .map(|elem| elem.mul(scalar, alloc))
            .collect();

        Ok(alloc.alloc(RuntimeTuple::from_vec(result?)))
    }
}

impl std::hash::Hash for RuntimeTuple<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for item in self.borrow().iter() {
            item.hash(state);
        }
    }
}
