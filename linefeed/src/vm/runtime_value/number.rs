use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum RuntimeNumber {
    SmallInt(isize),
    BigInt(Rc<rug::Integer>),
    Float(f64),
}

impl RuntimeNumber {
    pub fn floor_int(&self) -> isize {
        match self {
            SmallInt(i) => *i,
            BigInt(i) => i.to_isize().unwrap(),
            Float(f) => f.floor() as isize,
        }
    }

    pub fn to_i32(&self) -> Option<i32> {
        match self {
            SmallInt(i) => i32::try_from(*i).ok(),
            BigInt(i) => i.to_i32(),
            _ => None,
        }
    }

    pub fn floor(&self) -> Self {
        match self {
            SmallInt(i) => SmallInt(*i),
            BigInt(i) => BigInt(Rc::clone(i)),
            Float(f) => Float(f.floor()),
        }
    }

    pub fn bool(&self) -> bool {
        match self {
            SmallInt(i) => *i != 0,
            BigInt(i) => **i != 0,
            Float(f) => *f != 0.0,
        }
    }

    pub fn float(&self) -> f64 {
        match self {
            SmallInt(i) => *i as f64,
            BigInt(i) => i.to_f64(),
            Float(f) => *f,
        }
    }

    pub fn modulo(&self, other: &Self) -> Self {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => SmallInt(a % b),
            (SmallInt(a), BigInt(b)) => BigInt(Rc::new(rug::Integer::from(*a) % b.as_ref())),
            (SmallInt(a), Float(b)) => Float(*a as f64 % b),
            (BigInt(a), SmallInt(b)) => BigInt(Rc::new(a.as_ref() % rug::Integer::from(*b))),
            (BigInt(a), BigInt(b)) => BigInt(Rc::new((a.as_ref() % b.as_ref()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64() % b),
            (Float(a), SmallInt(b)) => Float(a % (*b as f64)),
            (Float(a), BigInt(b)) => Float(a % b.to_f64()),
            (Float(a), Float(b)) => Float(a % b),
        }
    }

    pub fn pow(&self, other: &Self) -> Self {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => {
                // For small int powers, try to stay in SmallInt range
                if let Ok(exp_u32) = u32::try_from(*b) {
                    if let Some(result) = a.checked_pow(exp_u32) {
                        return SmallInt(result);
                    }
                }
                // Overflow or negative exponent - promote to BigInt or Float
                if *b < 0 {
                    Float((*a as f64).powi(*b as i32))
                } else {
                    BigInt(Rc::new(rug::Integer::from(*a).pow(*b as u32)))
                }
            }
            (SmallInt(a), BigInt(b)) => {
                BigInt(Rc::new(rug::Integer::from(*a).pow(b.to_u32().unwrap())))
            }
            (SmallInt(a), Float(b)) => Float((*a as f64).powf(*b)),
            (BigInt(a), SmallInt(b)) => {
                if *b < 0 {
                    Float(a.to_f64().powi(*b as i32))
                } else {
                    BigInt(Rc::new(a.as_ref().pow(*b as u32).into()))
                }
            }
            (BigInt(a), BigInt(b)) => BigInt(Rc::new(a.as_ref().pow(b.to_u32().unwrap()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64().powf(*b)),
            (Float(a), SmallInt(b)) => Float(a.powi(*b as i32)),
            (Float(a), BigInt(b)) => Float(a.powi(b.to_i32().unwrap())),
            (Float(a), Float(b)) => Float(a.powf(*b)),
        }
    }

    pub fn div_floor(&self, other: &Self) -> Self {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => SmallInt(a / b),
            (SmallInt(a), BigInt(b)) => BigInt(Rc::new(rug::Integer::from(*a) / b.as_ref())),
            (SmallInt(a), Float(b)) => Float((*a as f64) / b).floor(),
            (BigInt(a), SmallInt(b)) => BigInt(Rc::new(a.as_ref() / rug::Integer::from(*b))),
            (BigInt(a), BigInt(b)) => BigInt(Rc::new((a.as_ref() / b.as_ref()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64() / b).floor(),
            (Float(a), SmallInt(b)) => Float(a / (*b as f64)).floor(),
            (Float(a), BigInt(b)) => Float(a / b.to_f64()).floor(),
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

    pub fn bitwise_and(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeNumber::SmallInt(a), RuntimeNumber::SmallInt(b)) => {
                Ok(RuntimeNumber::SmallInt(a & b))
            }
            (RuntimeNumber::SmallInt(a), RuntimeNumber::BigInt(b)) => Ok(RuntimeNumber::BigInt(
                Rc::new(rug::Integer::from(*a) & b.as_ref()),
            )),
            (RuntimeNumber::BigInt(a), RuntimeNumber::SmallInt(b)) => Ok(RuntimeNumber::BigInt(
                Rc::new(a.as_ref() & rug::Integer::from(*b)),
            )),
            (RuntimeNumber::BigInt(a), RuntimeNumber::BigInt(b)) => Ok(RuntimeNumber::BigInt(
                Rc::new((a.as_ref() & b.as_ref()).into()),
            )),
            _ => Err(RuntimeError::TypeMismatch(
                "Cannot use & on floating point numbers".to_string(),
            )),
        }
    }

    pub fn bitwise_or(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeNumber::SmallInt(a), RuntimeNumber::SmallInt(b)) => {
                Ok(RuntimeNumber::SmallInt(a | b))
            }
            (RuntimeNumber::SmallInt(a), RuntimeNumber::BigInt(b)) => Ok(RuntimeNumber::BigInt(
                Rc::new(rug::Integer::from(*a) | b.as_ref()),
            )),
            (RuntimeNumber::BigInt(a), RuntimeNumber::SmallInt(b)) => Ok(RuntimeNumber::BigInt(
                Rc::new(a.as_ref() | rug::Integer::from(*b)),
            )),
            (RuntimeNumber::BigInt(a), RuntimeNumber::BigInt(b)) => Ok(RuntimeNumber::BigInt(
                Rc::new((a.as_ref() | b.as_ref()).into()),
            )),
            _ => Err(RuntimeError::TypeMismatch(
                "Cannot use | on floating point numbers".to_string(),
            )),
        }
    }

    pub fn bitwise_xor(&self, other: &Self) -> Result<Self, RuntimeError> {
        match (self, other) {
            (RuntimeNumber::SmallInt(a), RuntimeNumber::SmallInt(b)) => {
                Ok(RuntimeNumber::SmallInt(a ^ b))
            }
            (RuntimeNumber::SmallInt(a), RuntimeNumber::BigInt(b)) => Ok(RuntimeNumber::BigInt(
                Rc::new(rug::Integer::from(*a) ^ b.as_ref()),
            )),
            (RuntimeNumber::BigInt(a), RuntimeNumber::SmallInt(b)) => Ok(RuntimeNumber::BigInt(
                Rc::new(a.as_ref() ^ rug::Integer::from(*b)),
            )),
            (RuntimeNumber::BigInt(a), RuntimeNumber::BigInt(b)) => Ok(RuntimeNumber::BigInt(
                Rc::new((a.as_ref() ^ b.as_ref()).into()),
            )),
            _ => Err(RuntimeError::TypeMismatch(
                "Cannot use ^ on floating point numbers".to_string(),
            )),
        }
    }

    pub fn bitwise_not(&self) -> Result<Self, RuntimeError> {
        match self {
            RuntimeNumber::SmallInt(a) => Ok(RuntimeNumber::SmallInt(!a)),
            RuntimeNumber::BigInt(a) => Ok(RuntimeNumber::BigInt(Rc::new((!a.as_ref()).into()))),
            RuntimeNumber::Float(_) => Err(RuntimeError::TypeMismatch(
                "Cannot use ~ on floating point numbers".to_string(),
            )),
        }
    }

    pub fn left_shift(&self, other: &Self) -> Result<Self, RuntimeError> {
        let shift_amount = other.to_shift_amount()?;

        match self {
            RuntimeNumber::SmallInt(a) => Ok(RuntimeNumber::SmallInt(a << shift_amount)),
            RuntimeNumber::BigInt(a) => Ok(RuntimeNumber::BigInt(Rc::new(
                (a.as_ref() << shift_amount).into(),
            ))),
            RuntimeNumber::Float(_) => Err(RuntimeError::TypeMismatch(
                "Cannot shift floating point numbers".to_string(),
            )),
        }
    }

    pub fn right_shift(&self, other: &Self) -> Result<Self, RuntimeError> {
        let shift_amount = other.to_shift_amount()?;

        match self {
            RuntimeNumber::SmallInt(a) => Ok(RuntimeNumber::SmallInt(a >> shift_amount)),
            RuntimeNumber::BigInt(a) => Ok(RuntimeNumber::BigInt(Rc::new(
                (a.as_ref() >> shift_amount).into(),
            ))),
            RuntimeNumber::Float(_) => Err(RuntimeError::TypeMismatch(
                "Cannot shift floating point numbers".to_string(),
            )),
        }
    }

    fn to_shift_amount(&self) -> Result<u32, RuntimeError> {
        match self {
            RuntimeNumber::SmallInt(b) if *b >= 0 => u32::try_from(*b)
                .map_err(|_| RuntimeError::Plain(format!("Shift amount too large: {b}"))),
            RuntimeNumber::SmallInt(b) => Err(RuntimeError::Plain(format!(
                "Cannot shift by negative amount: {b}",
            ))),
            RuntimeNumber::BigInt(b) => b
                .to_u32()
                .ok_or_else(|| RuntimeError::Plain(format!("Shift amount too large: {b}"))),
            RuntimeNumber::Float(_) => Err(RuntimeError::TypeMismatch(
                "Cannot shift by floating point amount".to_string(),
            )),
        }
    }

    pub fn binary(&self) -> Result<String, RuntimeError> {
        match self {
            RuntimeNumber::SmallInt(n) => Ok(format!("{n:b}")),
            RuntimeNumber::BigInt(n) => Ok(n.to_string_radix(2)),
            RuntimeNumber::Float(_) => Err(RuntimeError::TypeMismatch(
                "Cannot convert floating point numbers to binary".to_string(),
            )),
        }
    }

    pub fn neg(&self) -> Self {
        self * &RuntimeNumber::from(-1)
    }

    pub fn abs(&self) -> Self {
        match self {
            SmallInt(i) => SmallInt(i.abs()),
            BigInt(i) => BigInt(Rc::new(i.as_ref().clone().abs())),
            Float(f) => Float(f.abs()),
        }
    }
}

// Macro for types that always fit in isize
macro_rules! impl_smallint_from {
    ($($t:ty),*) => {
        $(
            impl From<$t> for RuntimeNumber {
                fn from(i: $t) -> Self {
                    SmallInt(i as isize)
                }
            }
        )*
    };
}

// Macro for types that might not fit in isize
macro_rules! impl_int_from {
    ($($t:ty),*) => {
        $(
            impl From<$t> for RuntimeNumber {
                fn from(i: $t) -> Self {
                    if let Ok(small) = isize::try_from(i) {
                        SmallInt(small)
                    } else {
                        BigInt(Rc::new(i.into()))
                    }
                }
            }
        )*
    };
}

// These types always fit in isize
impl_smallint_from!(i8, i16, i32, isize, u8, u16);

// These types might not fit in isize (depends on platform or size)
impl_int_from!(u32, i64, i128, u64, u128, usize);

impl std::fmt::Display for RuntimeNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SmallInt(i) => write!(f, "{}", i),
            BigInt(i) => write!(f, "{}", i),
            Float(fl) => write!(f, "{}", fl),
        }
    }
}

impl PartialEq for RuntimeNumber {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a == b,
            (SmallInt(a), BigInt(b)) => rug::integer::MiniInteger::from(*a) == **b,
            (SmallInt(a), Float(b)) => (*a as f64) == *b,
            (BigInt(a), SmallInt(b)) => **a == rug::integer::MiniInteger::from(*b),
            (BigInt(a), BigInt(b)) => a == b,
            (BigInt(a), Float(b)) => a.to_f64() == *b,
            (Float(a), SmallInt(b)) => *a == (*b as f64),
            (Float(a), BigInt(b)) => *a == b.to_f64(),
            (Float(a), Float(b)) => a == b,
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
            (SmallInt(a), SmallInt(b)) => a.partial_cmp(b),
            (SmallInt(a), BigInt(b)) => rug::Integer::from(*a).partial_cmp(b.as_ref()),
            (SmallInt(a), Float(b)) => (*a as f64).partial_cmp(b),
            (BigInt(a), SmallInt(b)) => a.as_ref().partial_cmp(&rug::Integer::from(*b)),
            (BigInt(a), BigInt(b)) => a.partial_cmp(b),
            (BigInt(a), Float(b)) => a.to_f64().partial_cmp(b),
            (Float(a), SmallInt(b)) => a.partial_cmp(&(*b as f64)),
            (Float(a), BigInt(b)) => a.partial_cmp(&b.to_f64()),
            (Float(a), Float(b)) => a.partial_cmp(b),
        }
    }
}

impl Add for RuntimeNumber {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a
                .checked_add(b)
                .map(SmallInt)
                .unwrap_or_else(|| BigInt(Rc::new(rug::Integer::from(a) + rug::Integer::from(b)))),
            (SmallInt(a), BigInt(b)) => BigInt(Rc::new(rug::Integer::from(a) + b.as_ref())),
            (SmallInt(a), Float(b)) => Float(a as f64 + b),
            (BigInt(a), SmallInt(b)) => BigInt(Rc::new(a.as_ref() + rug::Integer::from(b))),
            (BigInt(a), BigInt(b)) => BigInt(Rc::new((a.as_ref() + b.as_ref()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64() + b),
            (Float(a), SmallInt(b)) => Float(a + b as f64),
            (Float(a), BigInt(b)) => Float(a + b.to_f64()),
            (Float(a), Float(b)) => Float(a + b),
        }
    }
}

impl Add<&Self> for RuntimeNumber {
    type Output = Self;

    fn add(self, other: &Self) -> Self::Output {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a
                .checked_add(*b)
                .map(SmallInt)
                .unwrap_or_else(|| BigInt(Rc::new(rug::Integer::from(a) + rug::Integer::from(*b)))),
            (SmallInt(a), BigInt(b)) => BigInt(Rc::new(rug::Integer::from(a) + b.as_ref())),
            (SmallInt(a), Float(b)) => Float(a as f64 + b),
            (BigInt(a), SmallInt(b)) => BigInt(Rc::new(a.as_ref() + rug::Integer::from(*b))),
            (BigInt(a), BigInt(b)) => BigInt(Rc::new((a.as_ref() + b.as_ref()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64() + b),
            (Float(a), SmallInt(b)) => Float(a + *b as f64),
            (Float(a), BigInt(b)) => Float(a + b.to_f64()),
            (Float(a), Float(b)) => Float(a + b),
        }
    }
}

impl Add for &RuntimeNumber {
    type Output = RuntimeNumber;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a.checked_add(*b).map(SmallInt).unwrap_or_else(|| {
                BigInt(Rc::new(rug::Integer::from(*a) + rug::Integer::from(*b)))
            }),
            (SmallInt(a), BigInt(b)) => BigInt(Rc::new(rug::Integer::from(*a) + b.as_ref())),
            (SmallInt(a), Float(b)) => Float(*a as f64 + b),
            (BigInt(a), SmallInt(b)) => BigInt(Rc::new(a.as_ref() + rug::Integer::from(*b))),
            (BigInt(a), BigInt(b)) => BigInt(Rc::new((a.as_ref() + b.as_ref()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64() + b),
            (Float(a), SmallInt(b)) => Float(a + *b as f64),
            (Float(a), BigInt(b)) => Float(a + b.to_f64()),
            (Float(a), Float(b)) => Float(a + b),
        }
    }
}

impl Sub for RuntimeNumber {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a
                .checked_sub(b)
                .map(SmallInt)
                .unwrap_or_else(|| BigInt(Rc::new(rug::Integer::from(a) - rug::Integer::from(b)))),
            (SmallInt(a), BigInt(b)) => BigInt(Rc::new(rug::Integer::from(a) - b.as_ref())),
            (SmallInt(a), Float(b)) => Float(a as f64 - b),
            (BigInt(a), SmallInt(b)) => BigInt(Rc::new(a.as_ref() - rug::Integer::from(b))),
            (BigInt(a), BigInt(b)) => BigInt(Rc::new((a.as_ref() - b.as_ref()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64() - b),
            (Float(a), SmallInt(b)) => Float(a - b as f64),
            (Float(a), BigInt(b)) => Float(a - b.to_f64()),
            (Float(a), Float(b)) => Float(a - b),
        }
    }
}

impl Sub for &RuntimeNumber {
    type Output = RuntimeNumber;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a.checked_sub(*b).map(SmallInt).unwrap_or_else(|| {
                BigInt(Rc::new(rug::Integer::from(*a) - rug::Integer::from(*b)))
            }),
            (SmallInt(a), BigInt(b)) => BigInt(Rc::new(rug::Integer::from(*a) - b.as_ref())),
            (SmallInt(a), Float(b)) => Float(*a as f64 - b),
            (BigInt(a), SmallInt(b)) => BigInt(Rc::new(a.as_ref() - rug::Integer::from(*b))),
            (BigInt(a), BigInt(b)) => BigInt(Rc::new((a.as_ref() - b.as_ref()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64() - b),
            (Float(a), SmallInt(b)) => Float(a - *b as f64),
            (Float(a), BigInt(b)) => Float(a - b.to_f64()),
            (Float(a), Float(b)) => Float(a - b),
        }
    }
}

impl Mul for RuntimeNumber {
    type Output = RuntimeNumber;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a
                .checked_mul(b)
                .map(SmallInt)
                .unwrap_or_else(|| BigInt(Rc::new(rug::Integer::from(a) * rug::Integer::from(b)))),
            (SmallInt(a), BigInt(b)) => BigInt(Rc::new(rug::Integer::from(a) * b.as_ref())),
            (SmallInt(a), Float(b)) => Float(a as f64 * b),
            (BigInt(a), SmallInt(b)) => BigInt(Rc::new(a.as_ref() * rug::Integer::from(b))),
            (BigInt(a), BigInt(b)) => BigInt(Rc::new((a.as_ref() * b.as_ref()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64() * b),
            (Float(a), SmallInt(b)) => Float(a * b as f64),
            (Float(a), BigInt(b)) => Float(a * b.to_f64()),
            (Float(a), Float(b)) => Float(a * b),
        }
    }
}

impl Mul for &RuntimeNumber {
    type Output = RuntimeNumber;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a.checked_mul(*b).map(SmallInt).unwrap_or_else(|| {
                BigInt(Rc::new(rug::Integer::from(*a) * rug::Integer::from(*b)))
            }),
            (SmallInt(a), BigInt(b)) => BigInt(Rc::new(rug::Integer::from(*a) * b.as_ref())),
            (SmallInt(a), Float(b)) => Float(*a as f64 * b),
            (BigInt(a), SmallInt(b)) => BigInt(Rc::new(a.as_ref() * rug::Integer::from(*b))),
            (BigInt(a), BigInt(b)) => BigInt(Rc::new((a.as_ref() * b.as_ref()).into())),
            (BigInt(a), Float(b)) => Float(a.to_f64() * b),
            (Float(a), SmallInt(b)) => Float(a * *b as f64),
            (Float(a), BigInt(b)) => Float(a * b.to_f64()),
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
