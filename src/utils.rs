use core::num::NonZeroUsize;
use core::ops::{RangeBounds, RangeFrom, RangeInclusive};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncompleteInt<T> {
    Unbounded(RangeFrom<T>),
    Bounded(RangeInclusive<T>, NonZeroUsize),
}

impl<T> RangeBounds<T> for IncompleteInt<T> {
    fn start_bound(&self) -> std::collections::Bound<&T> {
        match self {
            IncompleteInt::Unbounded(range) => range.start_bound(),
            IncompleteInt::Bounded(range, _) => range.start_bound(),
        }
    }
    fn end_bound(&self) -> std::collections::Bound<&T> {
        match self {
            IncompleteInt::Unbounded(range) => range.end_bound(),
            IncompleteInt::Bounded(range, _) => range.end_bound(),
        }
    }
}

impl<T> IncompleteInt<T> {
    pub fn new_unbounded(start: T) -> Self {
        Self::Unbounded(RangeFrom { start })
    }

    pub fn new_bounded(range: (T, T), bits_needed: NonZeroUsize) -> Self {
        Self::Bounded(RangeInclusive::new(range.0, range.1), bits_needed)
    }
}
