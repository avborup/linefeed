use crate::vm::{
    runtime_value::{iterator::RuntimeIterator, list::RuntimeList, RuntimeValue},
    RuntimeError,
};

pub type RuntimeResult = Result<RuntimeValue, RuntimeError>;

pub fn all(args: Vec<RuntimeValue>) -> RuntimeResult {
    let iter = if let [arg] = args.as_slice() {
        match arg.to_iter_inner() {
            Ok(iter) => iter,
            Err(_) => return Ok(RuntimeValue::Bool(arg.bool())),
        }
    } else {
        RuntimeIterator::from(RuntimeList::from_vec(args))
    };

    while let Some(value) = iter.next() {
        if !value.bool() {
            return Ok(RuntimeValue::Bool(false));
        }
    }

    Ok(RuntimeValue::Bool(true))
}

pub fn any(args: Vec<RuntimeValue>) -> RuntimeResult {
    let iter = if let [arg] = args.as_slice() {
        match arg.to_iter_inner() {
            Ok(iter) => iter,
            Err(_) => return Ok(RuntimeValue::Bool(arg.bool())),
        }
    } else {
        RuntimeIterator::from(RuntimeList::from_vec(args))
    };

    while let Some(value) = iter.next() {
        if value.bool() {
            return Ok(RuntimeValue::Bool(true));
        }
    }

    Ok(RuntimeValue::Bool(false))
}
