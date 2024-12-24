use crate::{bytecode_interpreter::RuntimeError, runtime_value::RuntimeValue};

pub trait LfAppend {
    fn append(&mut self, other: RuntimeValue) -> Result<(), RuntimeError>;
}
