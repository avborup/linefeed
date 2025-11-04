use crate::vm::{
    runtime_value::{
        counter::RuntimeCounter, iterator::RuntimeIterator, list::RuntimeList, map::RuntimeMap,
        number::RuntimeNumber, set::RuntimeSet, tuple::RuntimeTuple, vector::RuntimeVector,
        RuntimeValue,
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

pub fn vec(x: RuntimeValue, y: RuntimeValue) -> RuntimeResult {
    let x_val = match x {
        RuntimeValue::Int(i) => i as f64,
        RuntimeValue::Num(n) => n.float(),
        _ => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Cannot create vector with x coordinate of type '{}'",
                x.kind_str()
            )))
        }
    };

    let y_val = match y {
        RuntimeValue::Int(i) => i as f64,
        RuntimeValue::Num(n) => n.float(),
        _ => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Cannot create vector with y coordinate of type '{}'",
                y.kind_str()
            )))
        }
    };

    Ok(RuntimeValue::Vector(RuntimeVector::new(x_val, y_val)))
}
