use std::{ops::Deref, rc::Rc};

use regex::Regex;

use crate::runtime_value::{list::RuntimeList, string::RuntimeString, RuntimeValue};

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

    pub fn find_matches(&self, s: &RuntimeString) -> RuntimeList {
        let s = s.as_str();

        let mut matches = Vec::new();

        for m in self.0.captures_iter(s) {
            let group_values = m
                .iter()
                .map(|group| {
                    group.map_or(RuntimeValue::Null, |g| {
                        RuntimeValue::Str(RuntimeString::new(g.as_str()))
                    })
                })
                .collect::<Vec<_>>();

            // TODO: Push tuples when they're implemented
            matches.push(RuntimeValue::List(RuntimeList::from_vec(group_values)));
        }

        RuntimeList::from_vec(matches)
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
