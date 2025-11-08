use std::mem::ManuallyDrop;

use oxc_allocator::Allocator;
use rug::{integer::MiniInteger, ops::Pow as _};

use RuntimeNumber::*;

#[derive(Debug, Clone)]
pub enum RuntimeNumber<'gc> {
    SmallInt(isize),
    BigInt(&'gc rug::Integer),
    Float(f64),
}

trait Allocated<'gc> {
    fn alloc(self, alloc: &'gc Allocator) -> &'gc Self;
}

impl<'gc> Allocated<'gc> for rug::Integer {
    fn alloc(self, alloc: &'gc Allocator) -> &'gc Self {
        alloc.alloc(ManuallyDrop::new(rug::Integer::from(10)))
    }
}

const _: () = {
    assert!(!std::mem::needs_drop::<RuntimeNumber>());
};

impl<'gc> RuntimeNumber<'gc> {
    pub fn floor_int(&self) -> isize {
        match self {
            SmallInt(i) => *i,
            BigInt(i) => i.to_isize().unwrap(),
            Float(f) => f.floor() as isize,
        }
    }

    pub fn floor(&self) -> Self {
        match self {
            SmallInt(i) => SmallInt(*i),
            BigInt(i) => BigInt(*i),
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

    pub fn modulo(&self, other: &Self, alloc: &'gc Allocator) -> Self {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => SmallInt(a % b),
            (SmallInt(a), BigInt(b)) => BigInt((rug::Integer::from(*a) % *b).alloc(alloc)),
            (SmallInt(a), Float(b)) => Float(*a as f64 % b),
            (BigInt(a), SmallInt(b)) => BigInt(rug::Integer::from(*a % *b).alloc(alloc)),
            (BigInt(a), BigInt(b)) => BigInt(rug::Integer::from(*a % *b).alloc(alloc)),
            (BigInt(a), Float(b)) => Float(a.to_f64() % b),
            (Float(a), SmallInt(b)) => Float(a % (*b as f64)),
            (Float(a), BigInt(b)) => Float(a % b.to_f64()),
            (Float(a), Float(b)) => Float(a % b),
        }
    }

    pub fn pow(&self, other: &Self, alloc: &'gc Allocator) -> Self {
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
                    BigInt(rug::Integer::from(*a).pow(*b as u32).alloc(alloc))
                }
            }
            (SmallInt(a), BigInt(b)) => {
                BigInt(rug::Integer::from(*a).pow(b.to_u32().unwrap()).alloc(alloc))
            }
            (SmallInt(a), Float(b)) => Float((*a as f64).powf(*b)),
            (BigInt(a), SmallInt(b)) => {
                if *b < 0 {
                    Float(a.to_f64().powi(*b as i32))
                } else {
                    BigInt(rug::Integer::from((*a).pow(*b as u32)).alloc(alloc))
                }
            }
            (BigInt(a), BigInt(b)) => {
                BigInt(rug::Integer::from((*a).pow(b.to_u32().unwrap())).alloc(alloc))
            }
            (BigInt(a), Float(b)) => Float(a.to_f64().powf(*b)),
            (Float(a), SmallInt(b)) => Float(a.powi(*b as i32)),
            (Float(a), BigInt(b)) => Float(a.powi(b.to_i32().unwrap())),
            (Float(a), Float(b)) => Float(a.powf(*b)),
        }
    }

    pub fn div_floor(&self, other: &Self, alloc: &'gc Allocator) -> Self {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => SmallInt(a / b),
            (SmallInt(a), BigInt(b)) => BigInt((rug::Integer::from(*a) / *b).alloc(alloc)),
            (SmallInt(a), Float(b)) => Float((*a as f64) / b).floor(),
            (BigInt(a), SmallInt(b)) => BigInt((*a / rug::Integer::from(*b)).alloc(alloc)),
            (BigInt(a), BigInt(b)) => BigInt(rug::Integer::from(*a / *b).alloc(alloc)),
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

    pub fn neg(&self, alloc: &'gc Allocator) -> Self {
        self.mul(&RuntimeNumber::from(-1), alloc)
    }

    pub fn add(&self, other: &Self, alloc: &'gc Allocator) -> Self {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a.checked_add(*b).map(SmallInt).unwrap_or_else(|| {
                BigInt((rug::Integer::from(*a) + rug::Integer::from(*b)).alloc(alloc))
            }),
            (SmallInt(a), BigInt(b)) => BigInt((rug::Integer::from(*a) + *b).alloc(alloc)),
            (SmallInt(a), Float(b)) => Float(*a as f64 + b),
            (BigInt(a), SmallInt(b)) => BigInt((*a + rug::Integer::from(*b)).alloc(alloc)),
            (BigInt(a), BigInt(b)) => BigInt(rug::Integer::from(*a + *b).alloc(alloc)),
            (BigInt(a), Float(b)) => Float(a.to_f64() + b),
            (Float(a), SmallInt(b)) => Float(a + *b as f64),
            (Float(a), BigInt(b)) => Float(a + b.to_f64()),
            (Float(a), Float(b)) => Float(a + b),
        }
    }

    pub fn sub(&self, other: &Self, alloc: &'gc Allocator) -> Self {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a.checked_sub(*b).map(SmallInt).unwrap_or_else(|| {
                BigInt((rug::Integer::from(*a) - rug::Integer::from(*b)).alloc(alloc))
            }),
            (SmallInt(a), BigInt(b)) => BigInt((rug::Integer::from(*a) - *b).alloc(alloc)),
            (SmallInt(a), Float(b)) => Float(*a as f64 - b),
            (BigInt(a), SmallInt(b)) => BigInt((*a - rug::Integer::from(*b)).alloc(alloc)),
            (BigInt(a), BigInt(b)) => BigInt(rug::Integer::from(*a - *b).alloc(alloc)),
            (BigInt(a), Float(b)) => Float(a.to_f64() - b),
            (Float(a), SmallInt(b)) => Float(a - *b as f64),
            (Float(a), BigInt(b)) => Float(a - b.to_f64()),
            (Float(a), Float(b)) => Float(a - b),
        }
    }

    pub fn mul(&self, other: &Self, alloc: &'gc Allocator) -> Self {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a.checked_mul(*b).map(SmallInt).unwrap_or_else(|| {
                BigInt((rug::Integer::from(*a) * rug::Integer::from(*b)).alloc(alloc))
            }),
            (SmallInt(a), BigInt(b)) => BigInt((rug::Integer::from(*a) * (*b)).alloc(alloc)),
            (SmallInt(a), Float(b)) => Float(*a as f64 * b),
            (BigInt(a), SmallInt(b)) => BigInt((*a * rug::Integer::from(*b)).alloc(alloc)),
            (BigInt(a), BigInt(b)) => BigInt(rug::Integer::from((*a) * (*b)).alloc(alloc)),
            (BigInt(a), Float(b)) => Float(a.to_f64() * b),
            (Float(a), SmallInt(b)) => Float(a * *b as f64),
            (Float(a), BigInt(b)) => Float(a * b.to_f64()),
            (Float(a), Float(b)) => Float(a * b),
        }
    }

    pub fn div(&self, other: &Self) -> Self {
        RuntimeNumber::Float(self.float() / other.float())
    }
}

// Macro for types that always fit in isize
macro_rules! impl_smallint_from {
    ($($t:ty),*) => {
        $(
            impl<'gc> From<$t> for RuntimeNumber<'gc> {
                fn from(i: $t) -> Self {
                    SmallInt(i as isize)
                }
            }
        )*
    };
}

pub trait FromWithAlloc<'gc, T> {
    fn from_with_alloc(i: T, alloc: &'gc Allocator) -> RuntimeNumber<'gc>;
}

// Macro for types that might not fit in isize
macro_rules! impl_int_from {
    ($($t:ty),*) => {
        $(
            impl<'gc> FromWithAlloc<'gc, $t> for RuntimeNumber<'gc> {
                fn from_with_alloc(i: $t, alloc: &'gc Allocator) -> Self {
                    if let Ok(small) = isize::try_from(i) {
                        SmallInt(small)
                    } else {
                        BigInt(rug::Integer::from(i).alloc(alloc))
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

impl std::fmt::Display for RuntimeNumber<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SmallInt(i) => write!(f, "{}", i),
            BigInt(i) => write!(f, "{}", i),
            Float(fl) => write!(f, "{}", fl),
        }
    }
}

impl PartialEq for RuntimeNumber<'_> {
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

use crate::vm::RuntimeError;

// Fuck it, we ball
impl Eq for RuntimeNumber<'_> {}

impl std::hash::Hash for RuntimeNumber<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // FIXME: This is just going to make hashing floats terrible, but it's a start.
        self.floor_int().hash(state)
    }
}

impl std::cmp::PartialOrd for RuntimeNumber<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SmallInt(a), SmallInt(b)) => a.partial_cmp(b),
            (SmallInt(a), BigInt(b)) => MiniInteger::from(*a).partial_cmp(*b),
            (SmallInt(a), Float(b)) => (*a as f64).partial_cmp(b),
            (BigInt(a), SmallInt(b)) => (*a).partial_cmp(&MiniInteger::from(*b)),
            (BigInt(a), BigInt(b)) => a.partial_cmp(b),
            (BigInt(a), Float(b)) => a.to_f64().partial_cmp(b),
            (Float(a), SmallInt(b)) => a.partial_cmp(&(*b as f64)),
            (Float(a), BigInt(b)) => a.partial_cmp(&b.to_f64()),
            (Float(a), Float(b)) => a.partial_cmp(b),
        }
    }
}
