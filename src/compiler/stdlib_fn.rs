use std::ops::RangeInclusive;

use crate::compiler::method::define_names;

#[derive(Debug, Clone)]
pub enum StdlibFn {
    Print,
    Input,
    ParseInt,
    Repr,
    ToList,
    ToTuple,
    ToMap,
    Product,
    Sum,
    All,
    Any,
    Max,
    Min,
}

impl StdlibFn {
    define_names! {
        Print => "print",
        Input => "input",
        ParseInt => "int",
        Repr => "repr",
        ToList => "list",
        ToTuple => "tuple",
        ToMap => "map",
        Product => "mul",
        Sum => "sum",
        All => "all",
        Any => "any",
        Max => "max",
        Min => "min",
    }

    /// Returns the number of arguments this function expects.
    pub fn num_args(&self) -> RangeInclusive<usize> {
        match self {
            Self::Print => 0..=usize::MAX,
            Self::Input => 0..=0, // TODO: in the future future, read from an optional file path here?
            Self::ParseInt => 1..=1,
            Self::Repr => 1..=1,
            Self::ToList => 1..=1,
            Self::ToTuple => 1..=1,
            Self::ToMap => 1..=1,
            Self::Product => 1..=1,
            Self::Sum => 1..=1,
            Self::All => 1..=usize::MAX,
            Self::Any => 1..=usize::MAX,
            Self::Max => 1..=usize::MAX,
            Self::Min => 1..=usize::MAX,
        }
    }
}
