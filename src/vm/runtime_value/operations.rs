use crate::vm::{runtime_value::RuntimeValue, RuntimeError};

pub trait LfAppend {
    fn append(&mut self, other: RuntimeValue) -> Result<(), RuntimeError>;
}
