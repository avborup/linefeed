use crate::vm::{runtime_value::RuntimeValue, RuntimeError};

pub trait LfAppend<'gc> {
    fn append(&mut self, other: RuntimeValue<'gc>) -> Result<(), RuntimeError>;
}
