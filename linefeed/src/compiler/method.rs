use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub enum Method {
    Append,
    Add,
    ToUpperCase,
    ToLowerCase,
    Split,
    SplitLines,
    Join,
    Length,
    Count,
    FindAll,
    Find,
    IsMatch,
    Contains,
    Sort,
    Enumerate,
}

impl Method {
    define_names! {
        Append => "append",
        Add => "add",
        ToUpperCase => "upper",
        ToLowerCase => "lower",
        Split => "split",
        SplitLines => "lines",
        Length => "len",
        Count => "count",
        FindAll => "find_all",
        Find => "find",
        IsMatch => "is_match",
        Join => "join",
        Contains => "contains",
        Sort => "sort",
        Enumerate => "enumerate",
    }

    /// Returns the number of arguments this method expects.
    pub fn num_args(&self) -> RangeInclusive<usize> {
        match self {
            Self::Append => 1..=1,
            Self::Add => 1..=1,
            Self::ToUpperCase => 0..=0,
            Self::ToLowerCase => 0..=0,
            Self::Split => 1..=1,
            Self::SplitLines => 0..=0,
            Self::Length => 0..=0,
            Self::Count => 1..=1,
            Self::FindAll => 1..=1,
            Self::Find => 1..=1,
            Self::IsMatch => 1..=1,
            Self::Join => 0..=1,
            Self::Contains => 1..=1,
            Self::Sort => 0..=0,
            Self::Enumerate => 0..=0,
        }
    }
}

macro_rules! define_names {
    ($($variant:ident => $name:expr),* $(,)?) => {
        pub fn name(&self) -> &'static str {
            match self {
                $(Self::$variant => $name),*
            }
        }

        pub fn from_name(name: &str) -> Option<Self> {
            match name {
                $($name => Some(Self::$variant)),*,
                _ => None,
            }
        }
    };
}

pub(crate) use define_names;
