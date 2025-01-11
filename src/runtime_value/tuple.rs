use std::cell::Ref;

use crate::{
    bytecode_interpreter::RuntimeError,
    runtime_value::{list::RuntimeList, number::RuntimeNumber, RuntimeValue},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct RuntimeTuple(RuntimeList);

impl RuntimeTuple {
    pub fn from_vec(vec: Vec<RuntimeValue>) -> Self {
        Self(RuntimeList::from_vec(vec))
    }

    pub fn as_slice(&self) -> Ref<'_, [RuntimeValue]> {
        self.0.as_slice()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn deep_clone(&self) -> Self {
        Self(self.0.deep_clone())
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue, RuntimeError> {
        self.0.index(index)
    }
}
