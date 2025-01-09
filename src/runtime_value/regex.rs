use std::{ops::Deref, rc::Rc};

use regex::Regex;

#[derive(Debug, Clone)]
pub struct RuntimeRegex(Rc<Regex>);

impl RuntimeRegex {
    pub fn new(regex: Regex) -> Self {
        Self(Rc::new(regex))
    }

    pub fn as_regex(&self) -> &Regex {
        &self.0
    }

    pub fn deep_clone(&self) -> Self {
        Self::new(self.0.deref().clone())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for RuntimeRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "/{}/", self.as_str())
    }
}

impl std::cmp::PartialEq for RuntimeRegex {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl std::cmp::Eq for RuntimeRegex {}

impl std::hash::Hash for RuntimeRegex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}
