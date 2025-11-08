use std::marker::PhantomData;

use oxc_allocator::Allocator;

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

    pub fn alloc(self, alloc: &Allocator) -> &Self {
        alloc.alloc(self)
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

        let lower = lower.is_none_or(|lower| value >= &RuntimeNumber::from(lower));
        let upper = upper.is_none_or(|upper| value < &RuntimeNumber::from(upper));

        lower && upper
    }
}

impl std::fmt::Display for RuntimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(start) = self.start {
            write!(f, "{}", start)?;
        }
        write!(f, "..")?;
        if let Some(end) = self.end {
            write!(f, "{}", end)?;
        }
        Ok(())
    }
}

pub struct RangeIterator<'gc> {
    range: RuntimeRange,
    value: isize,
    step: isize,
    phantom: PhantomData<&'gc ()>,
}

impl<'gc> RangeIterator<'gc> {
    pub fn new(range: RuntimeRange) -> Self {
        Self {
            value: range.start.unwrap_or(0),
            step: if range.is_reverse() { -1 } else { 1 },
            range,
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> Option<usize> {
        match (self.range.start, self.range.end) {
            (Some(start), Some(end)) => {
                let dist = start.abs_diff(end);
                Some(dist / self.step.unsigned_abs())
            }
            _ => None,
        }
    }
}

impl<'gc> Iterator for RangeIterator<'gc> {
    type Item = RuntimeValue<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.step.abs() == 1);

        let (value, step, end) = (self.value, self.step, self.range.end);

        if step.is_positive() && end.is_some_and(|end| value >= end)
            || step.is_negative() && end.is_some_and(|end| value <= end)
        {
            return None;
        }

        let output = self.value;
        self.value += step;

        Some(RuntimeValue::Num(RuntimeNumber::from(output)))
    }
}
