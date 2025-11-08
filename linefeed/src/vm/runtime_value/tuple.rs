use oxc_allocator::{Allocator, Vec as AVec};

use crate::vm::{
    runtime_value::{number::RuntimeNumber, utils::resolve_index, RuntimeValue},
    RuntimeError,
};

#[derive(Debug, PartialEq, Eq)]
// TODO: just use cell
pub struct RuntimeTuple<'gc>(AVec<'gc, RuntimeValue<'gc>>);

impl<'gc> RuntimeTuple<'gc> {
    pub fn from_iter(
        iter: impl IntoIterator<Item = RuntimeValue<'gc>>,
        alloc: &'gc Allocator,
    ) -> Self {
        Self(AVec::from_iter_in(iter, alloc))
    }

    pub fn from_vec(vec: AVec<'gc, RuntimeValue<'gc>>) -> Self {
        Self(vec)
    }

    pub fn alloc(self, alloc: &'gc Allocator) -> &'gc Self {
        alloc.alloc(self)
    }

    pub fn as_slice(&self) -> &[RuntimeValue<'gc>] {
        self.0.as_slice()
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue<'gc>, RuntimeError> {
        let i = resolve_index(self.len(), index)?;

        self.as_slice()
            .get(i)
            .cloned()
            .ok_or_else(|| RuntimeError::IndexOutOfBounds(i as isize, self.len()))
    }

    pub fn contains(&self, value: &RuntimeValue<'gc>) -> bool {
        self.as_slice().iter().any(|v| v == value)
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

        let result = self
            .as_slice()
            .iter()
            .zip(other.as_slice().iter())
            .map(|(a, b)| a.add(b, alloc))
            .try_fold(AVec::with_capacity_in(self.len(), alloc), |mut acc, res| {
                acc.push(res?);
                Ok(acc)
            })?;

        Ok(alloc.alloc(RuntimeTuple::from_vec(result)))
    }

    pub fn scalar_multiply(
        &self,
        scalar: &RuntimeValue<'gc>,
        alloc: &'gc Allocator,
    ) -> Result<&Self, RuntimeError> {
        let result = self
            .as_slice()
            .iter()
            .map(|elem| elem.mul(scalar, alloc))
            .try_fold(AVec::with_capacity_in(self.len(), alloc), |mut acc, res| {
                acc.push(res?);
                Ok(acc)
            })?;

        Ok(alloc.alloc(RuntimeTuple::from_vec(result)))
    }
}

impl<'old, 'new> oxc_allocator::CloneIn<'new> for RuntimeTuple<'old> {
    type Cloned = RuntimeTuple<'new>;

    fn clone_in(&self, alloc: &'new Allocator) -> Self::Cloned {
        let cloned = self.as_slice().iter().map(|v| v.clone_in(alloc)).fold(
            AVec::with_capacity_in(self.len(), alloc),
            |mut acc, v| {
                acc.push(v);
                acc
            },
        );

        RuntimeTuple::from_vec(cloned)
    }
}

impl std::cmp::PartialOrd for RuntimeTuple<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.as_slice();
        let b = other.as_slice();
        a.partial_cmp(b)
    }
}

impl std::hash::Hash for RuntimeTuple<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for item in self.as_slice().iter() {
            item.hash(state);
        }
    }
}
