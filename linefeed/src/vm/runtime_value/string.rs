use std::rc::Rc;

use oxc_allocator::{Allocator, Vec as AVec};

use crate::vm::{
    runtime_value::{
        list::RuntimeList,
        number::RuntimeNumber,
        range::RuntimeRange,
        utils::{resolve_index, resolve_slice_indices},
        RuntimeValue,
    },
    RuntimeError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeString<'gc>(&'gc str);

impl<'gc> RuntimeString<'gc> {
    pub fn from_str(s: &'gc str) -> Self {
        Self(s)
    }

    pub fn alloc_from_str(s: impl AsRef<str>, alloc: &'gc Allocator) -> &'gc Self {
        Self(alloc.alloc_str(s.as_ref())).alloc(alloc)
    }

    pub fn alloc(self, alloc: &'gc Allocator) -> &'gc Self {
        alloc.alloc(self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn map_str<A: AsRef<str>>(
        &self,
        f: impl FnOnce(&str) -> A,
        alloc: &'gc Allocator,
    ) -> &'gc Self {
        Self::alloc_from_str(f(self.as_str()), alloc)
    }

    pub fn to_lowercase(&self, alloc: &'gc Allocator) -> &'gc Self {
        self.map_str(|s| s.to_lowercase(), alloc)
    }

    pub fn to_uppercase(&self, alloc: &'gc Allocator) -> &'gc Self {
        self.map_str(|s| s.to_uppercase(), alloc)
    }

    pub fn split(&self, delimiter: &RuntimeString, alloc: &'gc Allocator) -> &'gc RuntimeList<'gc> {
        let parts = self
            .as_str()
            .split(delimiter.as_str())
            .map(|s| RuntimeValue::Str(Self::alloc_from_str(s, alloc)));

        RuntimeList::from_vec(AVec::from_iter_in(parts, alloc)).alloc(alloc)
    }

    pub fn lines(&self, alloc: &'gc Allocator) -> &'gc RuntimeList<'gc> {
        let parts = self
            .as_str()
            .lines()
            .map(|s| RuntimeValue::Str(Self::alloc_from_str(s, alloc)));

        RuntimeList::from_vec(AVec::from_iter_in(parts, alloc)).alloc(alloc)
    }

    pub fn concat(&self, other: impl AsRef<str>, alloc: &'gc Allocator) -> &'gc Self {
        Self::from_str(alloc.alloc_concat_strs_array([self.as_str(), other.as_ref()])).alloc(alloc)
    }

    pub fn count(&self, substr: &RuntimeString) -> RuntimeNumber<'gc> {
        let n = self.as_str().matches(substr.as_str()).count();
        RuntimeNumber::from(n as isize)
    }

    pub fn index(
        &self,
        index: &RuntimeNumber,
        alloc: &'gc Allocator,
    ) -> Result<&'gc Self, RuntimeError> {
        let i = resolve_index(self.len(), index)?;

        // Not quite the best for Rust's UTF-8 strings, but all inputs for Linefeed's use-cases
        // will be valid ASCII, so indexing into the bytes directly should be fine for now.
        let byte = self.as_str().as_bytes().get(i as usize).ok_or_else(|| {
            RuntimeError::InternalBug(format!(
                "Index {i} is out of bounds for string of length {}",
                self.len()
            ))
        })?;

        Ok(Self::alloc_from_str(char::from(*byte).to_string(), alloc))
    }

    pub fn contains(&self, substr: &RuntimeString) -> bool {
        self.as_str().contains(substr.as_str())
    }

    pub fn substr(
        &self,
        range: &RuntimeRange,
        alloc: &'gc Allocator,
    ) -> Result<&'gc Self, RuntimeError> {
        let (start, end) = resolve_slice_indices(self.len(), range)?;
        Ok(Self::alloc_from_str(&self.as_str()[start..end + 1], alloc))
    }
}

impl<'old, 'new> oxc_allocator::CloneIn<'new> for RuntimeString<'old> {
    type Cloned = RuntimeString<'new>;

    fn clone_in(&self, alloc: &'new Allocator) -> Self::Cloned {
        RuntimeString(alloc.alloc_str(self.as_str()))
    }
}

impl std::fmt::Display for RuntimeString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for RuntimeString<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

pub struct StringIterator<'gc> {
    pub chars: AVec<'gc, &'gc RuntimeString<'gc>>,
    index: usize,
}

impl<'gc> StringIterator<'gc> {
    pub fn new(s: &'gc RuntimeString<'gc>, alloc: &'gc Allocator) -> Self {
        let chars = s
            .as_str()
            .chars()
            .map(|ch| RuntimeString::alloc_from_str(ch.to_string(), alloc))
            .fold(AVec::with_capacity_in(s.len(), alloc), |mut vec, ch| {
                vec.push(ch);
                vec
            });

        Self { chars, index: 0 }
    }

    pub fn next(&mut self) -> Option<RuntimeValue<'gc>> {
        let value = self.chars.get(self.index).cloned()?;
        self.index += 1;
        Some(RuntimeValue::Str(value))
    }
}
