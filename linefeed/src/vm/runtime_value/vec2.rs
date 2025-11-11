use crate::vm::runtime_error::RuntimeError;
use crate::vm::runtime_value::{RuntimeNumber, RuntimeTuple, RuntimeValue};

/// Stack-allocated 2D vector optimized for small integer coordinates.
/// Gracefully falls back to RuntimeTuple when operations overflow or encounter type mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeVec2 {
    pub x: i32,
    pub y: i32,
}

macro_rules! unwrap_or_fallback {
    ($opt:expr, $self:expr, $method:ident $(, $arg:expr)*) => {
        match $opt {
            Some(res) => res,
            None => {
                let tuple = $self.to_tuple();
                return Ok(RuntimeValue::Tuple(tuple.$method($($arg),*)?));
            }
        }
    };
}

impl RuntimeVec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_tuple(&self) -> RuntimeTuple {
        RuntimeTuple::from_vec_inner(vec![
            RuntimeValue::Num(RuntimeNumber::SmallInt(self.x as isize)),
            RuntimeValue::Num(RuntimeNumber::SmallInt(self.y as isize)),
        ])
    }

    pub fn add(&self, other: &Self) -> Result<RuntimeValue, RuntimeError> {
        let res = self.x.checked_add(other.x).zip(self.y.checked_add(other.y));

        let (x, y) = unwrap_or_fallback!(res, self, element_wise_add, &other.to_tuple());

        Ok(RuntimeValue::Vec2(RuntimeVec2::new(x, y)))
    }

    pub fn sub(&self, other: &Self) -> Result<RuntimeValue, RuntimeError> {
        let res = self.x.checked_sub(other.x).zip(self.y.checked_sub(other.y));

        let (x, y) = unwrap_or_fallback!(res, self, element_wise_sub, &other.to_tuple());

        Ok(RuntimeValue::Vec2(RuntimeVec2::new(x, y)))
    }

    pub fn scalar_mul(&self, scalar: &RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
        let res = scalar
            .to_i32()
            .and_then(|s| self.x.checked_mul(s).zip(self.y.checked_mul(s)));

        let (x, y) = unwrap_or_fallback!(res, self, scalar_multiply, scalar);

        Ok(RuntimeValue::Vec2(RuntimeVec2::new(x, y)))
    }

    pub fn rot(&self, times: &RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
        let n = unwrap_or_fallback!(times.to_i32(), self, rot, times);

        let rotated = match n.rem_euclid(4) {
            0 => *self,                              // No rotation
            1 => RuntimeVec2::new(self.y, -self.x),  // 90 degrees clockwise
            2 => RuntimeVec2::new(-self.x, -self.y), // 180 degrees
            3 => RuntimeVec2::new(-self.y, self.x),  // 270 degrees clockwise (= 90 ccw)
            _ => unreachable!("rotations should only be 0-3"),
        };

        Ok(RuntimeValue::Vec2(rotated))
    }

    pub fn index(&self, index: &RuntimeNumber) -> Result<RuntimeValue, RuntimeError> {
        let idx = index
            .to_i32()
            .ok_or_else(|| RuntimeError::IndexOutOfBounds(index.floor_int(), 2))?;

        let normalized_idx = if idx < 0 { 2 + idx } else { idx };
        let val = match normalized_idx {
            0 => self.x,
            1 => self.y,
            _ => return Err(RuntimeError::IndexOutOfBounds(idx as isize, 2)),
        };

        Ok(RuntimeValue::Num(RuntimeNumber::SmallInt(val as isize)))
    }

    pub fn contains(&self, value: &RuntimeValue) -> bool {
        value
            .to_i32()
            .map(|v| self.x == v || self.y == v)
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        2
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::cmp::PartialOrd for RuntimeVec2 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for RuntimeVec2 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.x.cmp(&other.x) {
            std::cmp::Ordering::Equal => self.y.cmp(&other.y),
            other => other,
        }
    }
}

impl TryFrom<(RuntimeValue, RuntimeValue)> for RuntimeVec2 {
    type Error = ();

    fn try_from((v1, v2): (RuntimeValue, RuntimeValue)) -> Result<Self, Self::Error> {
        Self::try_from((&v1, &v2))
    }
}

impl TryFrom<(&RuntimeValue, &RuntimeValue)> for RuntimeVec2 {
    type Error = ();

    fn try_from((v1, v2): (&RuntimeValue, &RuntimeValue)) -> Result<Self, Self::Error> {
        v1.to_i32()
            .zip(v2.to_i32())
            .map(|(x, y)| RuntimeVec2::new(x, y))
            .ok_or(())
    }
}

impl From<(RuntimeValue, RuntimeValue)> for RuntimeValue {
    fn from((v1, v2): (RuntimeValue, RuntimeValue)) -> Self {
        RuntimeVec2::try_from((&v1, &v2))
            .map(RuntimeValue::Vec2)
            .unwrap_or_else(|_| RuntimeValue::Tuple(RuntimeTuple::from_vec_inner(vec![v1, v2])))
    }
}
