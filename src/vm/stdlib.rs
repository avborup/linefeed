use crate::vm::{
    runtime_value::{
        iterator::RuntimeIterator, list::RuntimeList, number::RuntimeNumber, RuntimeValue,
    },
    RuntimeError,
};

pub type RuntimeResult = Result<RuntimeValue, RuntimeError>;

pub fn sum(val: RuntimeValue) -> RuntimeResult {
    let Ok(RuntimeValue::Iterator(iter)) = val.to_iter() else {
        return Err(RuntimeError::TypeMismatch(format!(
            "Cannot sum over type {}",
            val.kind_str()
        )));
    };

    let mut sum = RuntimeValue::Num(RuntimeNumber::Int(0));
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

    let mut prod = RuntimeValue::Num(RuntimeNumber::Int(1));
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
