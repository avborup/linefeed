use std::cmp::Ordering;

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

pub fn all(args: Vec<RuntimeValue>) -> RuntimeResult {
    Ok(RuntimeValue::Bool(args.iter().all(|v| v.bool())))
}

pub fn any(args: Vec<RuntimeValue>) -> RuntimeResult {
    Ok(RuntimeValue::Bool(args.iter().any(|v| v.bool())))
}

pub fn max(args: Vec<RuntimeValue>) -> RuntimeResult {
    let max = args
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .cloned();

    max.ok_or_else(|| {
        RuntimeError::Plain("Received empty iterator, cannot find maximum".to_string())
    })
}

pub fn min(args: Vec<RuntimeValue>) -> RuntimeResult {
    let min = args
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .cloned();

    min.ok_or_else(|| {
        RuntimeError::Plain("Received empty iterator, cannot find minimum".to_string())
    })
}
