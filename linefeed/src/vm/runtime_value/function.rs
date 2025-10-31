use crate::vm::runtime_value::RuntimeValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeFunction<L = usize> {
    pub arity: usize,
    pub location: L,
    pub is_memoized: bool,
    // TODO: Support default arguments
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoizationKey {
    pub func_location: usize,
    pub args: Vec<RuntimeValue>,
}
