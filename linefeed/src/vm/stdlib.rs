use crate::vm::{
    runtime_value::{
        iterator::RuntimeIterator, list::RuntimeList, map::RuntimeMap, number::RuntimeNumber,
        set::RuntimeSet, tuple::RuntimeTuple, RuntimeValue,
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

pub fn to_map(val: RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
    if let RuntimeValue::Map(_) = val {
        return Ok(val.clone());
    }

    if let RuntimeValue::Function(func) = val {
        return Ok(RuntimeValue::Map(RuntimeMap::from_default_generator(func)));
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

pub fn sum(val: RuntimeValue) -> RuntimeResult {
    let Ok(RuntimeValue::Iterator(iter)) = val.to_iter() else {
        return Err(RuntimeError::TypeMismatch(format!(
            "Cannot sum over type {}",
            val.kind_str()
        )));
    };

    let mut sum = RuntimeValue::Num(RuntimeNumber::from(0));
    while let Some(val) = iter.next() {
        sum = sum.add(&val)?;
    }

    Ok(sum)
}

pub fn mul(val: RuntimeValue) -> RuntimeResult {
    let Ok(RuntimeValue::Iterator(iter)) = val.to_iter() else {
        return Err(RuntimeError::TypeMismatch(format!(
            "Cannot multiply over type {}",
            val.kind_str()
        )));
    };

    let mut prod = RuntimeValue::Num(RuntimeNumber::from(1));
    while let Some(val) = iter.next() {
        prod = prod.mul(&val)?;
    }

    Ok(prod)
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

    let mut max = None;
    while let Some(value) = iter.next() {
        max = match max {
            Some(max) => Some(if value > max { value } else { max }),
            None => Some(value),
        };
    }

    max.ok_or_else(|| {
        RuntimeError::Plain("Received empty iterator, cannot find maximum".to_string())
    })
}

pub fn min(args: Vec<RuntimeValue>) -> RuntimeResult {
    let iter = iterator_from_variadic_args(args);

    let mut min = None;
    while let Some(value) = iter.next() {
        min = match min {
            Some(min) => Some(if value < min { value } else { min }),
            None => Some(value),
        };
    }

    min.ok_or_else(|| {
        RuntimeError::Plain("Received empty iterator, cannot find minimum".to_string())
    })
}
