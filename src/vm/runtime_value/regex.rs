use std::{ops::Deref, rc::Rc};

use regex::{Regex, RegexBuilder};

use crate::vm::runtime_value::{
    list::RuntimeList, number::RuntimeNumber, string::RuntimeString, tuple::RuntimeTuple,
    RuntimeValue,
};

#[derive(Debug, Clone)]
pub struct RuntimeRegex(Rc<RegexConfig>);

#[derive(Debug, Clone)]
pub struct RegexConfig {
    pub regex: Regex,
    pub modifiers: RegexModifiers,
}

#[derive(Debug, Clone)]
pub struct RegexModifiers {
    pub case_insensitive: bool,

    // This is custom to linefeed, and not part of the regex crate.
    //
    // In competitive programming, and especially in Advent of Code, if a match is a valid integer,
    // the use-case is almost always to parse it as an integer afterwards. So we provide a shortcut
    // for that, which keeps Linefeed code cleaner.
    pub parse_nums: bool,
}

impl RuntimeRegex {
    pub fn compile(s: &str, modifiers: RegexModifiers) -> Result<Self, regex::Error> {
        let regex = RegexBuilder::new(s)
            .case_insensitive(modifiers.case_insensitive)
            .build()?;

        Ok(Self::new(RegexConfig { regex, modifiers }))
    }

    pub fn new(regex: RegexConfig) -> Self {
        Self(Rc::new(regex))
    }

    pub fn as_regex(&self) -> &Regex {
        &self.0.regex
    }

    pub fn deep_clone(&self) -> Self {
        Self::new(self.0.deref().clone())
    }

    pub fn as_str(&self) -> &str {
        self.0.regex.as_str()
    }

    pub fn find_matches(&self, s: &RuntimeString) -> RuntimeList {
        let s = s.as_str();

        let mut matches = Vec::new();

        for m in self.0.regex.captures_iter(s) {
            let mut group_values = m
                .iter()
                .map(|group| {
                    group.map_or(RuntimeValue::Null, |g| {
                        if self.0.modifiers.parse_nums {
                            if let Ok(num) = g.as_str().parse::<isize>() {
                                return RuntimeValue::Num(RuntimeNumber::Int(num));
                            }
                        }

                        RuntimeValue::Str(RuntimeString::new(g.as_str()))
                    })
                })
                .collect::<Vec<_>>();

            // The full match is almost never useful, so just put it at the end, enabling the user
            // to just ignore it if they don't need it.
            let full_match = group_values.remove(0);
            group_values.push(full_match);

            matches.push(RuntimeValue::Tuple(RuntimeTuple::from_vec(group_values)));
        }

        RuntimeList::from_vec(matches)
    }

    // TODO: add find (single) and matches
}

impl std::fmt::Display for RuntimeRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "/{}/", self.as_str())?;

        // Destructuring to enforce compile error if new modifiers are added.
        let RegexModifiers {
            case_insensitive,
            parse_nums,
        } = self.0.modifiers;

        if case_insensitive {
            write!(f, "i")?;
        }
        if parse_nums {
            write!(f, "n")?;
        }

        Ok(())
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
