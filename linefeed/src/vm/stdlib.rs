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

    Ok(RuntimeValue::Tuple(RuntimeTuple::from_vec(iter.to_vec())))
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
