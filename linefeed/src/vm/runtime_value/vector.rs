use std::fmt;

use crate::vm::{runtime_error::RuntimeError, runtime_value::RuntimeValue};

use super::number::RuntimeNumber;

/// A 2D vector with x and y coordinates
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RuntimeVector {
    x: f64,
    y: f64,
}

impl RuntimeVector {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn add(&self, other: &Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue, RuntimeError> {
        match index.floor_int() {
            0 => Ok(RuntimeValue::Num(RuntimeNumber::Float(self.x))),
            1 => Ok(RuntimeValue::Num(RuntimeNumber::Float(self.y))),
            i => Err(RuntimeError::IndexOutOfBounds(i, 2)),
        }
    }
}

impl fmt::Display for RuntimeVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Eq for RuntimeVector {}

impl std::hash::Hash for RuntimeVector {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}
