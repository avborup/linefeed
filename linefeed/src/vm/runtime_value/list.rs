use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use ahash::AHashMap;

use crate::vm::{
    runtime_value::{
        number::RuntimeNumber,
        operations::LfAppend,
        range::RuntimeRange,
        utils::{resolve_index, resolve_slice_indices},
        RuntimeValue,
    },
    RuntimeError,
};

#[derive(Debug, Clone)]
pub struct RuntimeList(Rc<RefCell<Vec<RuntimeValue>>>);

impl RuntimeList {
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    pub fn from_vec(vec: Vec<RuntimeValue>) -> Self {
        Self(Rc::new(RefCell::new(vec)))
    }

    pub fn as_slice(&self) -> Ref<'_, [RuntimeValue]> {
        Ref::map(self.0.borrow(), |v| v.as_slice())
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    pub fn deep_clone(&self) -> Self {
        Self::from_vec(self.0.borrow().iter().map(|v| v.deep_clone()).collect())
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue, RuntimeError> {
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
        value: RuntimeValue,
    ) -> Result<(), RuntimeError> {
        let i = resolve_index(self.len(), index)?;
        self.0.borrow_mut()[i] = value;
        Ok(())
    }

    pub fn contains(&self, value: &RuntimeValue) -> bool {
        self.0.borrow().contains(value)
    }

    pub fn slice(&self, range: &RuntimeRange) -> Result<Self, RuntimeError> {
        let (start, end) = resolve_slice_indices(self.len(), range)?;
        Ok(Self::from_vec(self.0.borrow()[start..end + 1].to_vec()))
    }

    pub fn sort(&self) {
        self.0
            .borrow_mut()
            .sort_by(|a, b| a.partial_cmp(b).expect("unhandled uncomparable value"));
    }

    pub fn sort_by_key(
        &self,
        mut key_fn: impl FnMut(&RuntimeValue) -> Result<RuntimeValue, RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let keys = self
            .0
            .borrow()
            .iter()
            .map(|item| {
                let key = key_fn(item)?;
                Ok((item.clone(), key))
            })
            .collect::<Result<AHashMap<RuntimeValue, RuntimeValue>, RuntimeError>>()?;

        self.0.borrow_mut().sort_by(|a, b| {
            let key_a = keys.get(a).expect("key not found for item a");
            let key_b = keys.get(b).expect("key not found for item b");
            key_a
                .partial_cmp(key_b)
                .expect("unhandled uncomparable key value")
        });

        Ok(())
    }

    pub fn concat(&self, other: &Self) -> Self {
        let mut new_vec = self.0.borrow().clone();
        new_vec.extend_from_slice(&other.0.borrow());
        Self::from_vec(new_vec)
    }
}

impl Default for RuntimeList {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for RuntimeList {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.borrow();
        let b = other.0.borrow();

        a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for RuntimeList {}

impl std::hash::Hash for RuntimeList {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

impl std::cmp::PartialOrd for RuntimeList {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.borrow().partial_cmp(&other.0.borrow())
    }
}

impl LfAppend for RuntimeList {
    fn append(&mut self, other: RuntimeValue) -> Result<(), RuntimeError> {
        self.0.borrow_mut().push(other.clone());
        Ok(())
    }
}
