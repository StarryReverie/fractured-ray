use std::ops::{Bound, RangeBounds};

use super::Val;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisRange((Bound<Val>, Bound<Val>));

impl DisRange {
    pub fn positive() -> Self {
        Self((Bound::Excluded(Val(0.0)), Bound::Unbounded))
    }

    pub fn inclusive(min: Val, max: Val) -> Self {
        Self((Bound::Included(min), Bound::Included(max)))
    }

    pub fn unbounded() -> Self {
        Self((Bound::Unbounded, Bound::Unbounded))
    }

    pub fn empty() -> Self {
        Self((Bound::Excluded(Val(0.0)), Bound::Excluded(Val(0.0))))
    }

    pub fn advance_start(self, offset: Val) -> Self {
        let start = match self.0.0 {
            Bound::Included(o) => Bound::Excluded(o + offset),
            Bound::Excluded(o) => Bound::Excluded(o + offset),
            Bound::Unbounded => Bound::Unbounded,
        };
        (start, self.0.1).into()
    }

    pub fn shrink_end(self, end: Val) -> Self {
        let end = match self.0.1 {
            b @ Bound::Included(o) if o < end => b,
            b @ Bound::Excluded(o) if o < end => b,
            _ => Bound::Excluded(end),
        };
        (self.0.0, end).into()
    }

    pub fn intersect(self, other: Self) -> Self {
        use Bound::*;
        let (self_start, self_end) = self.0;
        let (other_start, other_end) = other.0;

        let start = match (self_start, other_start) {
            (Included(a), Included(b)) => Included(Ord::max(a, b)),
            (Excluded(a), Excluded(b)) => Excluded(Ord::max(a, b)),
            (Unbounded, Unbounded) => Unbounded,
            (x, Unbounded) | (Unbounded, x) => x,
            (Included(i), Excluded(e)) | (Excluded(e), Included(i)) => {
                if i > e {
                    Included(i)
                } else {
                    Excluded(e)
                }
            }
        };
        let end = match (self_end, other_end) {
            (Included(a), Included(b)) => Included(Ord::min(a, b)),
            (Excluded(a), Excluded(b)) => Excluded(Ord::min(a, b)),
            (Unbounded, Unbounded) => Unbounded,
            (x, Unbounded) | (Unbounded, x) => x,
            (Included(i), Excluded(e)) | (Excluded(e), Included(i)) => {
                if i < e {
                    Included(i)
                } else {
                    Excluded(e)
                }
            }
        };

        Self((start, end))
    }

    pub fn not_empty(&self) -> bool {
        use Bound::*;
        match (self.start_bound(), self.end_bound()) {
            (Unbounded, _) | (_, Unbounded) => true,
            (Included(start), Excluded(end))
            | (Excluded(start), Included(end))
            | (Excluded(start), Excluded(end)) => start < end,
            (Included(start), Included(end)) => start <= end,
        }
    }
}

impl From<(Bound<Val>, Bound<Val>)> for DisRange {
    fn from(value: (Bound<Val>, Bound<Val>)) -> Self {
        Self(value)
    }
}

impl From<DisRange> for (Bound<Val>, Bound<Val>) {
    fn from(value: DisRange) -> Self {
        value.0
    }
}

impl RangeBounds<Val> for DisRange {
    fn start_bound(&self) -> Bound<&Val> {
        self.0.start_bound()
    }

    fn end_bound(&self) -> Bound<&Val> {
        self.0.end_bound()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dis_range_shrink_end_succeeds() {
        let range = DisRange::positive();
        assert_eq!(range.end_bound(), Bound::Unbounded);
        let range = range.shrink_end(Val(10.0));
        assert_eq!(range.end_bound(), Bound::Excluded(&Val(10.0)));
        let range = range.shrink_end(Val(20.0));
        assert_eq!(range.end_bound(), Bound::Excluded(&Val(10.0)));
    }
}
