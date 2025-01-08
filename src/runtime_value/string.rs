use std::rc::Rc;

use crate::runtime_value::{list::RuntimeList, RuntimeValue};

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

    pub fn concat(&self, other: &RuntimeString) -> Self {
        Self::new(format!("{}{}", self.as_str(), other.as_str()))
    }
}

impl std::fmt::Display for RuntimeString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
