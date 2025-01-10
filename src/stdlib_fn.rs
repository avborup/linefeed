use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub enum StdlibFn {
    Print,
    Input,
    ParseInt,
    Repr,
    ToList,
    Product,
    Sum,
}

impl StdlibFn {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Print => "print",
            Self::Input => "input",
            Self::ParseInt => "int",
            Self::Repr => "repr",
            Self::ToList => "list",
            Self::Product => "mul",
            Self::Sum => "sum",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "print" => Some(Self::Print),
            "input" => Some(Self::Input),
            "int" => Some(Self::ParseInt),
            "repr" => Some(Self::Repr),
            "list" => Some(Self::ToList),
            "mul" => Some(Self::Product),
            "sum" => Some(Self::Sum),
            _ => None,
        }
    }

    /// Returns the number of arguments this function expects.
    pub fn num_args(&self) -> RangeInclusive<usize> {
        match self {
            Self::Print => 0..=usize::MAX,
            Self::Input => 0..=0, // TODO: in the future future, read from an optional file path here?
            Self::ParseInt => 1..=1,
            Self::Repr => 1..=1,
            Self::ToList => 1..=1,
            Self::Product => 1..=1,
            Self::Sum => 1..=1,
        }
    }
}
