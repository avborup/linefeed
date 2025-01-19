use crate::vm::runtime_value::{number::RuntimeNumber, RuntimeValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeRange {
    pub start: Option<isize>,
    pub end: Option<isize>,
}

impl RuntimeRange {
    pub fn new(start: Option<RuntimeNumber>, end: Option<RuntimeNumber>) -> Self {
        Self {
            start: start.map(|n| n.floor_int()),
            end: end.map(|n| n.floor_int()),
        }
    }

    pub fn is_reverse(&self) -> bool {
        match (self.start, self.end) {
            (Some(start), Some(end)) => start > end,
            _ => false,
        }
    }

    pub fn contains(&self, value: &RuntimeNumber) -> bool {
        let (lower, upper) = if !self.is_reverse() {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        };

        let lower = lower.map_or(true, |lower| value >= &RuntimeNumber::Int(lower));
        let upper = upper.map_or(true, |upper| value < &RuntimeNumber::Int(upper));

        lower && upper
    }
}

impl std::fmt::Display for RuntimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.end {
            Some(end) => write!(f, "{:?}..{:?}", self.start, end),
            None => write!(f, "{:?}..", self.start),
        }
    }
}

pub struct RangeIterator {
    range: RuntimeRange,
    value: isize,
    step: isize,
}

impl RangeIterator {
    pub fn new(range: RuntimeRange) -> Self {
        Self {
            value: range.start.unwrap_or(0),
            step: if range.is_reverse() { -1 } else { 1 },
            range,
        }
    }
}

impl Iterator for RangeIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.step.abs() == 1);

        let (value, step, end) = (self.value, self.step, self.range.end);

        if step.is_positive() && end.map_or(false, |end| value >= end)
            || step.is_negative() && end.map_or(false, |end| value <= end)
        {
            return None;
        }

        let output = self.value;
        self.value += step;

        Some(RuntimeValue::Num(RuntimeNumber::Int(output)))
    }
}
