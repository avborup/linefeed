use std::rc::Rc;

use crate::vm::{
    runtime_value::{number::RuntimeNumber, utils::resolve_index, vec2::RuntimeVec2, RuntimeValue},
    RuntimeError,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct RuntimeTuple(Rc<Vec<RuntimeValue>>);

impl RuntimeTuple {
    pub fn from_vec(vec: Vec<RuntimeValue>) -> Self {
        Self(Rc::new(vec))
    }

    /// Creates a RuntimeValue from a vec, optimizing 2-element small integer tuples to Vec2
    pub fn from_vec_optimized(vec: Vec<RuntimeValue>) -> RuntimeValue {
        // Try to optimize to Vec2 if it's a 2-element tuple with small integers
        if vec.len() == 2 {
            if let (Some(v1), Some(v2)) = (vec.get(0), vec.get(1)) {
                if let Ok(vec2) = RuntimeVec2::try_from((v1.clone(), v2.clone())) {
                    return RuntimeValue::Vec2(vec2);
                }
            }
        }
        // Otherwise, create a regular tuple
        RuntimeValue::Tuple(Self::from_vec(vec))
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

    pub fn element_wise_sub(&self, other: &Self) -> Result<Self, RuntimeError> {
        if self.len() != other.len() {
            return Err(RuntimeError::TypeMismatch(format!(
                "Cannot subtract tuples of different lengths: {} and {}",
                self.len(),
                other.len()
            )));
        }

        let result: Result<Vec<RuntimeValue>, RuntimeError> = self
            .0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| a.sub(b))
            .collect();

        Ok(RuntimeTuple::from_vec(result?))
    }

    pub fn scalar_multiply(&self, scalar: &RuntimeValue) -> Result<Self, RuntimeError> {
        let result: Result<Vec<RuntimeValue>, RuntimeError> =
            self.0.iter().map(|elem| elem.mul(scalar)).collect();

        Ok(RuntimeTuple::from_vec(result?))
    }

    pub fn rot(&self, times: &RuntimeValue) -> Result<Self, RuntimeError> {
        let (x, y) = match self.as_slice() {
            [RuntimeValue::Num(x), RuntimeValue::Num(y)] => (x, y),
            [a, b] => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "rotation requires tuple elements to be numbers, got '{}' and '{}'",
                    a.kind_str(),
                    b.kind_str()
                )))
            }
            _ => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "rotation only works on 2D numeric tuples, but got tuple with {} elements",
                    self.len()
                )))
            }
        };

        let rotation_count = match times {
            RuntimeValue::Num(n) => n.floor_int(),
            other => {
                return Err(RuntimeError::TypeMismatch(format!(
                    "rotation requires a numeric argument, got '{}'",
                    other.kind_str()
                )))
            }
        };

        let normalized = ((rotation_count % 4) + 4) % 4;

        let (new_x, new_y) = match normalized {
            0 => (x.clone(), y.clone()), // 0 degrees: no rotation
            1 => (y.clone(), x.neg()),   // 90 degrees clockwise
            2 => (x.neg(), y.neg()),     // 180 degrees
            3 => (y.neg(), x.clone()),   // 270 degrees clockwise (= 90 counter-clockwise)
            _ => unreachable!(),
        };

        Ok(RuntimeTuple::from_vec(vec![
            RuntimeValue::Num(new_x),
            RuntimeValue::Num(new_y),
        ]))
    }
}
