use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone)]
pub enum RuntimeNumber {
    // TODO: Arbitrary big integers. Reconsider the Copy trait in this case.
    Int(rug::Integer),
    Float(f64),
}

impl RuntimeNumber {
    pub fn floor_int(&self) -> isize {
        match self {
            Int(i) => i.to_isize().unwrap(),
            Float(f) => f.floor() as isize,
        }
    }

    pub fn floor(&self) -> Self {
        match self {
            Int(i) => Int(i.clone()),
            Float(f) => Float(f.floor()),
        }
    }

    pub fn bool(&self) -> bool {
        match self {
            Int(i) => *i != 0,
            Float(f) => *f != 0.0,
        }
    }

    pub fn float(&self) -> f64 {
        match self {
            Int(i) => i.to_f64(),
            Float(f) => *f,
        }
    }

    pub fn modulo(&self, other: &Self) -> Self {
        match (self, other) {
            (Int(a), Int(b)) => Int((a % b).into()),
            (Int(a), Float(b)) => Float(a.to_f64() % b),
            (Float(a), Int(b)) => Float(a % b.to_f64()),
            (Float(a), Float(b)) => Float(a % b),
        }
    }

    pub fn pow(&self, other: &Self) -> Self {
        match (self, other) {
            (Int(a), Int(b)) => Int(a.pow(b.to_u32().unwrap()).into()),
            (Int(a), Float(b)) => Float(a.to_f64().powf(*b)),
            (Float(a), Int(b)) => Float(a.powi(b.to_i32().unwrap())),
            (Float(a), Float(b)) => Float(a.powf(*b)),
        }
    }

    pub fn div_floor(&self, other: &Self) -> Self {
        match (self, other) {
            (Int(a), Int(b)) => Int((a / b).into()),
            (Int(a), Float(b)) => Float(a.to_f64() / b).floor(),
            (Float(a), Int(b)) => Float(a / b.to_f64()).floor(),
            (Float(a), Float(b)) => Float(a / b).floor(),
        }
    }

    pub fn parse_int(s: &str) -> Result<Self, RuntimeError> {
        match s.parse::<isize>() {
            Ok(i) => Ok(Self::from(i)),
            Err(err) => Err(RuntimeError::ParseError(format!(
                "{s:?} is not a valid integer, {err}",
            ))),
        }
    }
}

macro_rules! impl_bigint_from {
    ($($t:ty),*) => {
        $(
            impl From<$t> for RuntimeNumber {
                fn from(i: $t) -> Self {
                    Int(i.into())
                }
            }
        )*
    };
}

impl_bigint_from!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

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
            (Int(a), Float(b)) => a.to_f64() == *b,
            (Float(a), Int(b)) => *a == b.to_f64(),
        }
    }
}

use rug::ops::Pow as _;
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
            (Int(a), Float(b)) => a.to_f64().partial_cmp(b),
            (Float(a), Int(b)) => a.partial_cmp(&b.to_f64()),
        }
    }
}

impl Add for RuntimeNumber {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Int(a), Int(b)) => Int(a + b),
            (Int(a), Float(b)) => Float(a.to_f64() + b),
            (Float(a), Int(b)) => Float(a + b.to_f64()),
            (Float(a), Float(b)) => Float(a + b),
        }
    }
}

impl Add<&Self> for RuntimeNumber {
    type Output = Self;

    fn add(self, other: &Self) -> Self::Output {
        match (self, other) {
            (Int(a), Int(b)) => Int(a + b),
            (Int(a), Float(b)) => Float(a.to_f64() + b),
            (Float(a), Int(b)) => Float(a + b.to_f64()),
            (Float(a), Float(b)) => Float(a + b),
        }
    }
}

impl Add for &RuntimeNumber {
    type Output = RuntimeNumber;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Int(a), Int(b)) => Int((a + b).into()),
            (Int(a), Float(b)) => Float(a.to_f64() + b),
            (Float(a), Int(b)) => Float(a + b.to_f64()),
            (Float(a), Float(b)) => Float(a + b),
        }
    }
}

impl Sub for RuntimeNumber {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Int(a), Int(b)) => Int(a - b),
            (Int(a), Float(b)) => Float(a.to_f64() - b),
            (Float(a), Int(b)) => Float(a - b.to_f64()),
            (Float(a), Float(b)) => Float(a - b),
        }
    }
}

impl Sub for &RuntimeNumber {
    type Output = RuntimeNumber;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Int(a), Int(b)) => Int((a - b).into()),
            (Int(a), Float(b)) => Float(a.to_f64() - b),
            (Float(a), Int(b)) => Float(a - b.to_f64()),
            (Float(a), Float(b)) => Float(a - b),
        }
    }
}

impl Mul for RuntimeNumber {
    type Output = RuntimeNumber;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Int(a), Int(b)) => Int(a * b),
            (Int(a), Float(b)) => Float(a.to_f64() * b),
            (Float(a), Int(b)) => Float(a * b.to_f64()),
            (Float(a), Float(b)) => Float(a * b),
        }
    }
}

impl Mul for &RuntimeNumber {
    type Output = RuntimeNumber;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Int(a), Int(b)) => Int((a * b).into()),
            (Int(a), Float(b)) => Float(a.to_f64() * b),
            (Float(a), Int(b)) => Float(a * b.to_f64()),
            (Float(a), Float(b)) => Float(a * b),
        }
    }
}

impl Div for RuntimeNumber {
    type Output = RuntimeNumber;

    fn div(self, other: Self) -> Self::Output {
        RuntimeNumber::Float(self.float() / other.float())
    }
}

impl Div for &RuntimeNumber {
    type Output = RuntimeNumber;

    fn div(self, other: Self) -> Self::Output {
        RuntimeNumber::Float(self.float() / other.float())
    }
}
