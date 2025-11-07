use crate::{compiler::method::Method, vm::runtime_value::RuntimeValue};

#[derive(Debug, Clone)]
pub enum RuntimeError {
    Plain(String),
    StackUnderflow,
    NotImplemented(String),
    InvalidAddress(&'static str),
    TypeMismatch(String),
    InternalBug(String),
    IndexOutOfBounds(isize, usize),
    ParseError(String),
}

impl RuntimeError {
    pub fn invalid_binary_op_for_types(
        action: &str,
        lhs: &RuntimeValue,
        rhs: &RuntimeValue,
    ) -> Self {
        RuntimeError::TypeMismatch(format!(
            "Cannot {action} types '{}' and '{}'",
            lhs.kind_str(),
            rhs.kind_str()
        ))
    }

    pub fn invalid_method_for_type(method: Method, val: &RuntimeValue) -> Self {
        RuntimeError::TypeMismatch(format!(
            "Cannot call method '{}' on type '{}'",
            method.name(),
            val.kind_str()
        ))
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeError::Plain(msg) => write!(f, "{msg}"),
            RuntimeError::StackUnderflow => write!(f, "Stack underflow"),
            RuntimeError::NotImplemented(instr) => {
                write!(f, "Instruction not implemented: {instr}")
            }
            RuntimeError::InvalidAddress(kind_str) => {
                write!(f, "Invalid address of type {}", kind_str)
            }
            RuntimeError::TypeMismatch(msg) => {
                write!(f, "Type mismatch: {msg}")
            }
            RuntimeError::InternalBug(msg) => {
                write!(f, "Internal bug: {msg}")
            }
            RuntimeError::IndexOutOfBounds(i, len) => {
                write!(f, "Index {i} out of bounds, length is {len}")
            }
            RuntimeError::ParseError(msg) => {
                write!(f, "Parse error: {msg}")
            }
        }
    }
}
