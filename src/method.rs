#[derive(Debug, Clone)]
pub enum Method {
    Append,
    ToUpperCase,
    ToLowerCase,
    Split,
    SplitLines,
    Join,
    Length,
    Count,
    FindAll,
}

impl Method {
    pub fn name(&self) -> &'static str {
        match self {
            Method::Append => "append",
            Method::ToUpperCase => "upper",
            Method::ToLowerCase => "lower",
            Method::Split => "split",
            Method::SplitLines => "lines",
            Method::Length => "len",
            Method::Count => "count",
            Method::FindAll => "find_all",
            Method::Join => "join",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "append" => Some(Method::Append),
            "upper" => Some(Method::ToUpperCase),
            "lower" => Some(Method::ToLowerCase),
            "split" => Some(Method::Split),
            "lines" => Some(Method::SplitLines),
            "len" => Some(Method::Length),
            "count" => Some(Method::Count),
            "find_all" => Some(Method::FindAll),
            "join" => Some(Method::Join),
            _ => None,
        }
    }

    /// Returns the number of arguments this method expects or `None` if it is variadic.
    pub fn num_args(&self) -> Option<usize> {
        match self {
            Self::Append => Some(1),
            Self::ToUpperCase => Some(0),
            Self::ToLowerCase => Some(0),
            Self::Split => Some(1),
            Self::SplitLines => Some(0),
            Self::Length => Some(0),
            Self::Count => Some(1),
            Self::FindAll => Some(1),
            Self::Join => Some(1),
        }
    }
}
