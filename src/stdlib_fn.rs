#[derive(Debug, Clone)]
pub enum StdlibFn {
    Print,
}

impl StdlibFn {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Print => "print",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "print" => Some(Self::Print),
            _ => None,
        }
    }

    /// Returns the number of arguments this function expects or `None` if it is variadic.
    pub fn num_args(&self) -> Option<usize> {
        match self {
            Self::Print => None,
        }
    }
}
