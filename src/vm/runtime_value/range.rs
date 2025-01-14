use crate::vm::runtime_value::{number::RuntimeNumber, RuntimeValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeRange {
    pub start: isize,
    pub end: Option<isize>,
    pub step: isize,
}

impl RuntimeRange {
    pub fn new(start: RuntimeNumber, end: Option<RuntimeNumber>) -> Self {
        let is_reverse = end.as_ref().map_or(false, |end| start > *end);

        Self {
            start: start.floor_int(),
            end: end.map(|n| n.floor_int()),
            step: if is_reverse { -1 } else { 1 },
        }
    }

    pub fn contains(&self, value: &RuntimeNumber) -> bool {
        let (lower, upper) = if self.step.is_positive() {
            (Some(self.start), self.end)
        } else {
            (self.end, Some(self.start))
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
}

impl RangeIterator {
    pub fn new(range: RuntimeRange) -> Self {
        Self {
            value: range.start,
            range,
        }
    }
}

impl Iterator for RangeIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.range.step.abs() == 1);

        let (value, step, end) = (self.value, self.range.step, self.range.end);

        if step.is_positive() && end.map_or(false, |end| value >= end)
            || step.is_negative() && end.map_or(false, |end| value <= end)
        {
            return None;
        }

        let output = self.value;
        self.value += step;

        Some(RuntimeValue::Num(RuntimeNumber::Float(output as f64)))
    }
}
