use std::{ops::Deref, rc::Rc};

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
pub struct RuntimeString(Rc<String>);

impl RuntimeString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(Rc::new(s.into()))
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

    pub fn deep_clone(&self) -> Self {
        Self::new(self.0.deref().clone())
    }

    fn map_str(&self, f: impl FnOnce(&str) -> String) -> Self {
        Self::new(f(self.as_str()))
    }

    pub fn to_lowercase(&self) -> Self {
        self.map_str(|s| s.to_lowercase())
    }

    pub fn to_uppercase(&self) -> Self {
        self.map_str(|s| s.to_uppercase())
    }

    pub fn split(&self, delimiter: &RuntimeString) -> RuntimeList {
        let parts = self
            .as_str()
            .split(delimiter.as_str())
            .map(|s| RuntimeValue::Str(Self::new(s)))
            .collect();

        RuntimeList::from_vec(parts)
    }

    pub fn lines(&self) -> RuntimeList {
        let parts = self
            .as_str()
            .lines()
            .map(|s| RuntimeValue::Str(Self::new(s)))
            .collect();

        RuntimeList::from_vec(parts)
    }

    pub fn concat(&self, other: &RuntimeString) -> Self {
        Self::new(format!("{}{}", self.as_str(), other.as_str()))
    }

    pub fn count(&self, substr: &RuntimeString) -> RuntimeNumber {
        let n = self.as_str().matches(substr.as_str()).count();
        RuntimeNumber::from(n as isize)
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeString, RuntimeError> {
        let i = resolve_index(self.len(), index)?;

        // Not quite the best for Rust's UTF-8 strings, but all inputs for Linefeed's use-cases
        // will be valid ASCII, so indexing into the bytes directly should be fine for now.
        let byte = self.as_str().as_bytes().get(i as usize).ok_or_else(|| {
            RuntimeError::InternalBug(format!(
                "Index {i} is out of bounds for string of length {}",
                self.len()
            ))
        })?;

        Ok(Self::new(char::from(*byte)))
    }

    pub fn contains(&self, substr: &RuntimeString) -> bool {
        self.as_str().contains(substr.as_str())
    }

    pub fn substr(&self, range: &RuntimeRange) -> Result<Self, RuntimeError> {
        let (start, end) = resolve_slice_indices(self.len(), range)?;
        Ok(Self::new(&self.as_str()[start..end + 1]))
    }
}

impl std::fmt::Display for RuntimeString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
