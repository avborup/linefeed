#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeFunction<L = usize> {
    pub arity: usize,
    pub location: L,
    // TODO: Support default arguments
}
