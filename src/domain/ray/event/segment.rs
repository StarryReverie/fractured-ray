use std::ops::{Bound, RangeBounds};

use getset::CopyGetters;

use crate::domain::math::numeric::{DisRange, Val};

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RaySegment {
    start: Val,
    length: Val,
}

impl RaySegment {
    pub fn new(start: Val, length: Val) -> Self {
        Self { start, length }
    }

    pub fn end(&self) -> Val {
        self.start + self.length
    }

    pub fn contains(&self, distance: Val) -> bool {
        (self.start..=(self.start + self.length)).contains(&distance)
    }

    pub fn intersect(&self, rhs: &Self) -> Option<Self> {
        let (front, back) = if self.start < rhs.start {
            (self, rhs)
        } else {
            (rhs, self)
        };
        let end = front.start + front.length;
        if end > back.start {
            Some(Self::new(rhs.start, end - back.start))
        } else {
            None
        }
    }
}

impl From<DisRange> for RaySegment {
    fn from(value: DisRange) -> Self {
        let start = match value.start_bound() {
            Bound::Included(v) | Bound::Excluded(v) => *v,
            Bound::Unbounded => -Val::INFINITY,
        };
        let end = match value.end_bound() {
            Bound::Included(v) | Bound::Excluded(v) => *v,
            Bound::Unbounded => Val::INFINITY,
        };
        Self::new(start, end - start)
    }
}
