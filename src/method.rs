use std::ops::RangeInclusive;

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

    /// Returns the number of arguments this method expects.
    pub fn num_args(&self) -> RangeInclusive<usize> {
        match self {
            Self::Append => 1..=1,
            Self::ToUpperCase => 0..=0,
            Self::ToLowerCase => 0..=0,
            Self::Split => 1..=1,
            Self::SplitLines => 0..=0,
            Self::Length => 0..=0,
            Self::Count => 1..=1,
            Self::FindAll => 1..=1,
            Self::Join => 0..=1,
        }
    }
}
