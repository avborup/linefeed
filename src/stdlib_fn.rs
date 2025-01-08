#[derive(Debug, Clone)]
pub enum StdlibFn {
    Print,
    Input,
}

impl StdlibFn {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Print => "print",
            Self::Input => "input",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "print" => Some(Self::Print),
            "input" => Some(Self::Input),
            _ => None,
        }
    }

    /// Returns the number of arguments this function expects or `None` if it is variadic.
    pub fn num_args(&self) -> Option<usize> {
        match self {
            Self::Print => None,
            Self::Input => Some(0), // TODO: in the future future, read from an optional file path here?
        }
    }
}
