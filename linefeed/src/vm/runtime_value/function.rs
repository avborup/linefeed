use oxc_allocator::Allocator;

use crate::vm::runtime_value::RuntimeValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeFunction<L = usize> {
    pub arity: usize,
    pub location: L,
    pub is_memoized: bool,
    // TODO: Support default arguments
}

impl<'gc, L> RuntimeFunction<L> {
    pub fn alloc(self, alloc: &'gc Allocator) -> &'gc Self {
        alloc.alloc(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoizationKey<'gc> {
    pub func_location: usize,
    pub args: Vec<RuntimeValue<'gc>>,
}
