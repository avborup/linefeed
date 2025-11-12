use crate::vm::{
    runtime_value::{
        counter::RuntimeCounter, iterator::RuntimeIterator, list::RuntimeList, map::RuntimeMap,
        number::RuntimeNumber, set::RuntimeSet, tuple::RuntimeTuple, RuntimeValue,
    },
    RuntimeError,
};

pub type RuntimeResult = Result<RuntimeValue, RuntimeError>;

pub fn parse_int(val: RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
    let res = match val {
        RuntimeValue::Num(n) => RuntimeValue::Num(n.floor()),
        RuntimeValue::Str(s) => RuntimeValue::Num(RuntimeNumber::parse_int(s.as_str())?),
        _ => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Cannot parse '{}' as integer",
                val.kind_str()
            )))
        }
    };

    Ok(res)
}

pub fn to_list(val: RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
    if let RuntimeValue::List(_) = val {
        return Ok(val.clone());
    }

    let Ok(RuntimeValue::Iterator(iter)) = val.to_iter() else {
        return Err(RuntimeError::TypeMismatch(format!(
            "Cannot convert type {} to a list",
            val.kind_str()
        )));
    };

    Ok(RuntimeValue::List(RuntimeList::from_vec(iter.to_vec())))
}

pub fn to_tuple(val: RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
    if let RuntimeValue::Tuple(_) = val {
        return Ok(val.clone());
    }

    let Ok(RuntimeValue::Iterator(iter)) = val.to_iter() else {
        return Err(RuntimeError::TypeMismatch(format!(
            "Cannot convert type {} to a tuple",
            val.kind_str()
        )));
    };

    Ok(RuntimeTuple::from_vec(iter.to_vec()))
}

pub fn map_with_default(default_value: RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
    Ok(RuntimeValue::Map(RuntimeMap::new_with_default_value(
        default_value,
    )))
}

pub fn to_map(val: RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
    if let RuntimeValue::Map(_) = val {
        return Ok(val.clone());
    }

    let Ok(RuntimeValue::Iterator(iter)) = val.to_iter() else {
        return Err(RuntimeError::TypeMismatch(format!(
            "Cannot convert type {} to a map",
            val.kind_str()
        )));
    };

    Ok(RuntimeValue::Map(RuntimeMap::try_from(*iter)?))
}

pub fn to_set(val: Option<RuntimeValue>) -> Result<RuntimeValue, RuntimeError> {
    let iter = match val.as_ref().map(|v| v.to_iter_inner()) {
        None => RuntimeIterator::from(()),
        Some(Ok(iter)) => iter,
        Some(Err(_)) => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Cannot convert type {} to a set",
                val.unwrap().kind_str()
            )))
        }
    };

    Ok(RuntimeValue::Set(RuntimeSet::try_from(iter)?))
}

pub fn to_counter(val: Option<RuntimeValue>) -> Result<RuntimeValue, RuntimeError> {
    let iter = match val.as_ref().map(|v| v.to_iter_inner()) {
        None => RuntimeIterator::from(()),
        Some(Ok(iter)) => iter,
        Some(Err(_)) => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Cannot convert type {} to a counter",
                val.unwrap().kind_str()
            )))
        }
    };

    Ok(RuntimeValue::Counter(RuntimeCounter::try_from(iter)?))
}

pub fn sum(val: RuntimeValue) -> RuntimeResult {
    let Ok(RuntimeValue::Iterator(iter)) = val.to_iter() else {
        return Err(RuntimeError::TypeMismatch(format!(
            "Cannot sum over type {}",
            val.kind_str()
        )));
    };

    iter.try_fold(RuntimeValue::Num(RuntimeNumber::from(0)), |acc, v| {
        acc.add(&v)
    })
}

pub fn mul(val: RuntimeValue) -> RuntimeResult {
    let Ok(RuntimeValue::Iterator(iter)) = val.to_iter() else {
        return Err(RuntimeError::TypeMismatch(format!(
            "Cannot multiply over type {}",
            val.kind_str()
        )));
    };

    iter.try_fold(RuntimeValue::Num(RuntimeNumber::from(1)), |acc, v| {
        acc.mul(&v)
    })
}

fn iterator_from_variadic_args(args: Vec<RuntimeValue>) -> RuntimeIterator {
    if let [arg] = args.as_slice() {
        match arg.to_iter_inner() {
            Ok(iter) => iter,
            Err(_) => RuntimeIterator::from(RuntimeList::from_vec(args)),
        }
    } else {
        RuntimeIterator::from(RuntimeList::from_vec(args))
    }
}

pub fn all(args: Vec<RuntimeValue>) -> RuntimeResult {
    let iter = iterator_from_variadic_args(args);

    while let Some(value) = iter.next() {
        if !value.bool() {
            return Ok(RuntimeValue::Bool(false));
        }
    }

    Ok(RuntimeValue::Bool(true))
}

pub fn any(args: Vec<RuntimeValue>) -> RuntimeResult {
    let iter = iterator_from_variadic_args(args);

    while let Some(value) = iter.next() {
        if value.bool() {
            return Ok(RuntimeValue::Bool(true));
        }
    }

    Ok(RuntimeValue::Bool(false))
}

pub fn max(args: Vec<RuntimeValue>) -> RuntimeResult {
    let iter = iterator_from_variadic_args(args);

    let first = iter.next().ok_or_else(|| {
        RuntimeError::Plain("Received empty iterator, cannot find maximum".to_string())
    })?;

    Ok(iter.fold(first, |max, value| if value > max { value } else { max }))
}

pub fn min(args: Vec<RuntimeValue>) -> RuntimeResult {
    let iter = iterator_from_variadic_args(args);

    let first = iter.next().ok_or_else(|| {
        RuntimeError::Plain("Received empty iterator, cannot find minimum".to_string())
    })?;

    Ok(iter.fold(first, |min, value| if value < min { value } else { min }))
}

pub fn abs(val: RuntimeValue) -> RuntimeResult {
    match val {
        RuntimeValue::Num(n) => Ok(RuntimeValue::Num(n.abs())),
        _ => Err(RuntimeError::TypeMismatch(format!(
            "Cannot compute absolute value of type {}",
            val.kind_str()
        ))),
    }
}

pub fn manhattan(args: Vec<RuntimeValue>) -> RuntimeResult {
    let diff = match (args.first(), args.get(1)) {
        (Some(a), None) => a.clone(),
        (Some(a), Some(b)) => a.sub(b)?,
        (None, _) => unreachable!("manhattan function called with no arguments"),
    };

    let diff = match diff {
        RuntimeValue::Tuple(t) => t,
        RuntimeValue::Vec2(v) => v.to_tuple(),
        _ => {
            return Err(RuntimeError::TypeMismatch(format!(
                "cannot calculate manhattan distance for arguments of types: {}",
                args.iter()
                    .map(|v| v.kind_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )))
        }
    };

    let sum = diff
        .as_slice()
        .iter()
        .try_fold(RuntimeNumber::from(0), |acc, value| {
            Ok(acc + value.unwrap_num()?.abs())
        })?;

    Ok(RuntimeValue::Num(sum))
}

pub fn mod_inv(args: Vec<RuntimeValue>) -> RuntimeResult {
    let (Some(a_val), Some(m_val)) = (args.first(), args.get(1)) else {
        return Err(RuntimeError::Plain(
            "mod_inv requires exactly 2 arguments".to_string(),
        ));
    };

    let RuntimeValue::Num(a) = a_val else {
        return Err(RuntimeError::TypeMismatch(format!(
            "mod_inv first argument must be a number, got {}",
            a_val.kind_str()
        )));
    };

    let RuntimeValue::Num(m) = m_val else {
        return Err(RuntimeError::TypeMismatch(format!(
            "mod_inv second argument must be a number, got {}",
            m_val.kind_str()
        )));
    };

    // Extended Euclidean Algorithm
    // First normalize 'a' into the range [0, m)
    let m_abs = m.abs();
    let mut a_normalized = a.modulo(&m_abs);
    let zero = RuntimeNumber::from(0);
    if a_normalized < zero {
        a_normalized = &a_normalized + &m_abs;
    }

    let mut a = a_normalized;
    let mut b = m_abs.clone();
    let mut x0 = RuntimeNumber::from(0);
    let mut x1 = RuntimeNumber::from(1);
    let one = RuntimeNumber::from(1);

    if b == one {
        return Ok(RuntimeValue::Num(one));
    }

    while a > one {
        if b == zero {
            return Err(RuntimeError::Plain(
                "Modular inverse does not exist (gcd is not 1)".to_string(),
            ));
        }

        let q = a.div_floor(&b);

        let temp_a = b.clone();
        let temp_b = a.modulo(&b);
        a = temp_a;
        b = temp_b;

        let temp_x0 = &x1 - &(&q * &x0);
        let temp_x1 = x0;
        x0 = temp_x0;
        x1 = temp_x1;
    }

    if a != one {
        return Err(RuntimeError::Plain(
            "Modular inverse does not exist (gcd is not 1)".to_string(),
        ));
    }

    if x1 < zero {
        x1 = x1 + &m_abs;
    }

    // Adjust for negative modulus
    if m < &zero {
        x1 = x1.neg();
    }

    Ok(RuntimeValue::Num(x1))
}
