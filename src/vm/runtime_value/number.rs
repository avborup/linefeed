use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub enum RuntimeNumber {
    // TODO: Arbitrary big integers. Reconsider the Copy trait in this case.
    Int(isize),
    Float(f64),
}

impl RuntimeNumber {
    pub fn floor_int(&self) -> isize {
        match self {
            Int(i) => *i,
            Float(f) => f.floor() as isize,
        }
    }

    pub fn floor(&self) -> Self {
        Self::Float(self.float().floor())
    }

    pub fn bool(&self) -> bool {
        match self {
            Int(i) => *i != 0,
            Float(f) => *f != 0.0,
        }
    }

    pub fn float(&self) -> f64 {
        match self {
            Int(i) => *i as f64,
            Float(f) => *f,
        }
    }

    pub fn modulo(&self, other: &Self) -> Self {
        match (self, other) {
            (Int(a), Int(b)) => Int(a % b),
            (Int(a), Float(b)) => Float((*a as f64) % b),
            (Float(a), Int(b)) => Float(a % (*b as f64)),
            (Float(a), Float(b)) => Float(a % b),
        }
    }

    pub fn parse_int(s: &str) -> Result<Self, RuntimeError> {
        match s.parse::<isize>() {
            Ok(i) => Ok(Float(i as f64)),
            Err(err) => Err(RuntimeError::ParseError(format!(
                "{s:?} is not a valid integer, {err}",
            ))),
        }
    }
}

impl std::fmt::Display for RuntimeNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Int(i) => write!(f, "{}", i),
            Float(fl) => write!(f, "{}", fl),
        }
    }
}

impl PartialEq for RuntimeNumber {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Int(a), Float(b)) => (*a as f64) == *b,
            (Float(a), Int(b)) => *a == (*b as f64),
        }
    }
}

use RuntimeNumber::*;

use crate::vm::RuntimeError;

// Fuck it, we ball
impl Eq for RuntimeNumber {}

impl std::hash::Hash for RuntimeNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // FIXME: This is just going to make hashing floats terrible, but it's a start.
        self.floor_int().hash(state)
    }
}

impl std::cmp::PartialOrd for RuntimeNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Int(a), Int(b)) => a.partial_cmp(b),
            (Float(a), Float(b)) => a.partial_cmp(b),
            (Int(a), Float(b)) => (*a as f64).partial_cmp(b),
            (Float(a), Int(b)) => a.partial_cmp(&(*b as f64)),
        }
    }
}

impl Add for RuntimeNumber {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Int(a), Int(b)) => Int(a + b),
            (Int(a), Float(b)) => Float(a as f64 + b),
            (Float(a), Int(b)) => Float(a + b as f64),
            (Float(a), Float(b)) => Float(a + b),
        }
    }
}

impl Sub for RuntimeNumber {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Int(a), Int(b)) => Int(a - b),
            (Int(a), Float(b)) => Float(a as f64 - b),
            (Float(a), Int(b)) => Float(a - b as f64),
            (Float(a), Float(b)) => Float(a - b),
        }
    }
}

impl Mul for RuntimeNumber {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Int(a), Int(b)) => Int(a * b),
            (Int(a), Float(b)) => Float(a as f64 * b),
            (Float(a), Int(b)) => Float(a * b as f64),
            (Float(a), Float(b)) => Float(a * b),
        }
    }
}

impl Div for RuntimeNumber {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        RuntimeNumber::Float(self.float() / other.float())
    }
}
