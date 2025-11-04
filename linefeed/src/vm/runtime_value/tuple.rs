use std::rc::Rc;

use crate::vm::{
    runtime_value::{number::RuntimeNumber, utils::resolve_index, RuntimeValue},
    RuntimeError,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct RuntimeTuple(Rc<Vec<RuntimeValue>>);

impl RuntimeTuple {
    pub fn from_vec(vec: Vec<RuntimeValue>) -> Self {
        Self(Rc::new(vec))
    }

    pub fn as_slice(&self) -> &[RuntimeValue] {
        self.0.as_slice()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue, RuntimeError> {
        let i = resolve_index(self.len(), index)?;

        self.0
            .get(i)
            .cloned()
            .ok_or_else(|| RuntimeError::IndexOutOfBounds(i as isize, self.len()))
    }

    pub fn contains(&self, value: &RuntimeValue) -> bool {
        self.0.iter().any(|v| v == value)
    }

    pub fn element_wise_add(&self, other: &Self) -> Result<Self, RuntimeError> {
        if self.len() != other.len() {
            return Err(RuntimeError::TypeMismatch(format!(
                "Cannot add tuples of different lengths: {} and {}",
                self.len(),
                other.len()
            )));
        }

        let result: Result<Vec<RuntimeValue>, RuntimeError> = self
            .0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| a.add(b))
            .collect();

        Ok(RuntimeTuple::from_vec(result?))
    }
}
