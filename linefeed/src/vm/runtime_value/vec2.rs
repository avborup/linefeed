use crate::vm::runtime_error::RuntimeError;
use crate::vm::runtime_value::{RuntimeNumber, RuntimeTuple, RuntimeValue};

/// Stack-allocated 2D vector optimized for small integer coordinates.
/// Gracefully falls back to RuntimeTuple when operations overflow or encounter type mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeVec2 {
    pub x: isize,
    pub y: isize,
}

impl RuntimeVec2 {
    /// Creates a new Vec2
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    /// Converts this Vec2 to a RuntimeTuple
    pub fn to_tuple(&self) -> RuntimeTuple {
        RuntimeTuple::from_vec(vec![
            RuntimeValue::Num(RuntimeNumber::SmallInt(self.x)),
            RuntimeValue::Num(RuntimeNumber::SmallInt(self.y)),
        ])
    }

    /// Attempts to add two Vec2 values, falling back to tuple addition on overflow
    pub fn add(&self, other: &Self) -> RuntimeValue {
        match (self.x.checked_add(other.x), self.y.checked_add(other.y)) {
            (Some(x), Some(y)) => RuntimeValue::Vec2(RuntimeVec2::new(x, y)),
            _ => {
                // Overflow occurred, fall back to tuple addition
                let t1 = self.to_tuple();
                let t2 = other.to_tuple();
                // element_wise_add cannot fail for valid tuples of same length
                RuntimeValue::Tuple(t1.element_wise_add(&t2).unwrap())
            }
        }
    }

    /// Attempts to subtract two Vec2 values, falling back to tuple subtraction on overflow
    pub fn sub(&self, other: &Self) -> RuntimeValue {
        match (self.x.checked_sub(other.x), self.y.checked_sub(other.y)) {
            (Some(x), Some(y)) => RuntimeValue::Vec2(RuntimeVec2::new(x, y)),
            _ => {
                // Overflow occurred, fall back to tuple subtraction
                let t1 = self.to_tuple();
                let t2 = other.to_tuple();
                RuntimeValue::Tuple(t1.element_wise_sub(&t2).unwrap())
            }
        }
    }

    /// Attempts to multiply Vec2 by a scalar, falling back to tuple multiplication on overflow
    pub fn scalar_mul(&self, scalar: &RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
        // Extract the scalar value
        let RuntimeValue::Num(num) = scalar else {
            // Not a number, fall back to tuple
            let tuple = self.to_tuple();
            return Ok(RuntimeValue::Tuple(tuple.scalar_multiply(scalar)?));
        };

        match num {
            RuntimeNumber::SmallInt(s) => {
                // Try checked multiplication
                match (self.x.checked_mul(*s), self.y.checked_mul(*s)) {
                    (Some(new_x), Some(new_y)) => Ok(RuntimeValue::Vec2(RuntimeVec2::new(new_x, new_y))),
                    _ => {
                        // Overflow, fall back to tuple
                        let tuple = self.to_tuple();
                        Ok(RuntimeValue::Tuple(tuple.scalar_multiply(scalar)?))
                    }
                }
            }
            _ => {
                // BigInt or Float, fall back to tuple
                let tuple = self.to_tuple();
                Ok(RuntimeValue::Tuple(tuple.scalar_multiply(scalar)?))
            }
        }
    }

    /// Attempts to divide Vec2 by a scalar, falling back to tuple division on overflow or non-integer result
    pub fn scalar_div(&self, scalar: &RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
        // Extract the scalar value
        let RuntimeValue::Num(num) = scalar else {
            // Not a number, convert to tuple and delegate
            let tuple = self.to_tuple();
            let tuple_val = RuntimeValue::Tuple(tuple);
            return tuple_val.div(scalar);
        };

        match num {
            RuntimeNumber::SmallInt(s) => {
                if *s == 0 {
                    return Err(RuntimeError::Plain("Division by zero".to_string()));
                }
                // Try checked division - only succeeds if result is exact
                if self.x % s == 0 && self.y % s == 0 {
                    Ok(RuntimeValue::Vec2(RuntimeVec2::new(self.x / s, self.y / s)))
                } else {
                    // Non-integer result, fall back to tuple (which will produce floats)
                    let tuple = self.to_tuple();
                    let tuple_val = RuntimeValue::Tuple(tuple);
                    tuple_val.div(scalar)
                }
            }
            _ => {
                // BigInt or Float, fall back to tuple
                let tuple = self.to_tuple();
                let tuple_val = RuntimeValue::Tuple(tuple);
                tuple_val.div(scalar)
            }
        }
    }

    /// Attempts to compute Vec2 modulo a scalar
    pub fn scalar_rem(&self, scalar: &RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
        // Extract the scalar value
        let RuntimeValue::Num(RuntimeNumber::SmallInt(s)) = scalar else {
            // Not a small int, fall back to tuple
            let tuple = self.to_tuple();
            let tuple_val = RuntimeValue::Tuple(tuple);
            return tuple_val.modulo(scalar);
        };

        if *s == 0 {
            return Err(RuntimeError::Plain("Division by zero".to_string()));
        }

        Ok(RuntimeValue::Vec2(RuntimeVec2::new(self.x % s, self.y % s)))
    }

    /// Rotates this Vec2 by 90-degree increments (clockwise)
    /// Same logic as RuntimeTuple::rot but optimized for Vec2
    pub fn rot(&self, times: &RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
        let RuntimeValue::Num(RuntimeNumber::SmallInt(n)) = times else {
            // Not a small int, fall back to tuple
            let tuple = self.to_tuple();
            return tuple.rot(times).map(RuntimeValue::Tuple);
        };

        // Normalize to 0-3 range (4 rotations = 360 degrees = identity)
        let normalized = n.rem_euclid(4);

        match normalized {
            0 => Ok(RuntimeValue::Vec2(*self)),           // No rotation
            1 => Ok(RuntimeValue::Vec2(RuntimeVec2::new(self.y, -self.x))),  // 90 degrees clockwise
            2 => Ok(RuntimeValue::Vec2(RuntimeVec2::new(-self.x, -self.y))), // 180 degrees
            3 => Ok(RuntimeValue::Vec2(RuntimeVec2::new(-self.y, self.x))),  // 270 degrees clockwise (= 90 ccw)
            _ => unreachable!("rem_euclid(4) should only return 0-3"),
        }
    }

    /// Gets an element from Vec2 by index
    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue, RuntimeError> {
        let RuntimeNumber::SmallInt(idx) = index else {
            return Err(RuntimeError::TypeMismatch("Invalid index type".to_string()));
        };

        // Support negative indexing
        let normalized_idx = if *idx < 0 {
            2 + idx
        } else {
            *idx
        };

        match normalized_idx {
            0 => Ok(RuntimeValue::Num(RuntimeNumber::SmallInt(self.x))),
            1 => Ok(RuntimeValue::Num(RuntimeNumber::SmallInt(self.y))),
            _ => Err(RuntimeError::IndexOutOfBounds(*idx, 2)),
        }
    }

    /// Checks if Vec2 contains a value
    pub fn contains(&self, value: &RuntimeValue) -> bool {
        let RuntimeValue::Num(RuntimeNumber::SmallInt(v)) = value else {
            return false;
        };
        self.x == *v || self.y == *v
    }

    /// Compares two Vec2 values lexicographically
    pub fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.x.cmp(&other.x) {
            std::cmp::Ordering::Equal => self.y.cmp(&other.y),
            other => other,
        }
    }

    /// Returns the length (number of elements) of this Vec2
    pub fn len(&self) -> usize {
        2
    }
}

/// Try to create a RuntimeVec2 from two RuntimeValues
/// Returns None if either value is not a SmallInt
impl TryFrom<(RuntimeValue, RuntimeValue)> for RuntimeVec2 {
    type Error = ();

    fn try_from((v1, v2): (RuntimeValue, RuntimeValue)) -> Result<Self, Self::Error> {
        match (v1, v2) {
            (RuntimeValue::Num(RuntimeNumber::SmallInt(x)), RuntimeValue::Num(RuntimeNumber::SmallInt(y))) => {
                Ok(RuntimeVec2::new(x, y))
            }
            _ => Err(()),
        }
    }
}

/// Create a RuntimeValue from two RuntimeValues
/// Tries to create a Vec2 if both are SmallInts, otherwise creates a Tuple
impl From<(RuntimeValue, RuntimeValue)> for RuntimeValue {
    fn from((v1, v2): (RuntimeValue, RuntimeValue)) -> Self {
        // Try to create Vec2 first
        if let Ok(vec2) = RuntimeVec2::try_from((v1.clone(), v2.clone())) {
            RuntimeValue::Vec2(vec2)
        } else {
            // Fall back to tuple
            RuntimeValue::Tuple(RuntimeTuple::from_vec(vec![v1, v2]))
        }
    }
}
