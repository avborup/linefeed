use std::ops::RangeInclusive;

use crate::compiler::method::define_names;

#[derive(Debug, Clone)]
pub enum StdlibFn {
    Print,
    Input,
    ParseInt,
    Repr,
    Stringify,
    ToList,
    ToTuple,
    ToMap,
    MapWithDefault,
    ToSet,
    Product,
    Sum,
    All,
    Any,
    Max,
    Min,
    Abs,
    Counter,
    Manhattan,
    ModInv,
}

impl StdlibFn {
    define_names! {
        Print => "print",
        Input => "input",
        ParseInt => "int",
        Repr => "repr",
        Stringify => "str",
        ToList => "list",
        ToTuple => "tuple",
        ToMap => "map",
        MapWithDefault => "defaultmap",
        ToSet => "set",
        Product => "mul",
        Sum => "sum",
        All => "all",
        Any => "any",
        Max => "max",
        Min => "min",
        Abs => "abs",
        Counter => "counter",
        Manhattan => "manhattan",
        ModInv => "mod_inv",
    }

    /// Returns the number of arguments this function expects.
    pub fn num_args(&self) -> RangeInclusive<usize> {
        match self {
            Self::Print => 0..=usize::MAX,
            Self::Input => 0..=0, // TODO: in the future future, read from an optional file path here?
            Self::ParseInt => 1..=1,
            Self::Repr => 1..=1,
            Self::Stringify => 1..=1,
            Self::ToList => 1..=1,
            Self::ToTuple => 1..=1,
            Self::ToMap => 1..=1,
            Self::MapWithDefault => 1..=1,
            Self::ToSet => 0..=1,
            Self::Product => 1..=1,
            Self::Sum => 1..=1,
            Self::All => 1..=usize::MAX,
            Self::Any => 1..=usize::MAX,
            Self::Max => 1..=usize::MAX,
            Self::Min => 1..=usize::MAX,
            Self::Abs => 1..=1,
            Self::Counter => 0..=1,
            Self::Manhattan => 1..=2,
            Self::ModInv => 2..=2,
        }
    }
}
