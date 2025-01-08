#[derive(Debug, Clone)]
pub enum Method {
    Append,
    ToUpperCase,
    ToLowerCase,
}

impl Method {
    pub fn name(&self) -> &'static str {
        match self {
            Method::Append => "append",
            Method::ToUpperCase => "upper",
            Method::ToLowerCase => "lower",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "append" => Some(Method::Append),
            "upper" => Some(Method::ToUpperCase),
            "lower" => Some(Method::ToLowerCase),
            _ => None,
        }
    }

    /// Returns the number of arguments this function expects or `None` if it is variadic.
    pub fn num_args(&self) -> Option<usize> {
        match self {
            Self::Append => Some(1),
            Self::ToUpperCase => Some(0),
            Self::ToLowerCase => Some(0),
        }
    }
}
