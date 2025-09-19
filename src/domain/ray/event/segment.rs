use std::ops::{Bound, RangeBounds};

use getset::CopyGetters;

use crate::domain::math::geometry::Distance;
use crate::domain::math::numeric::DisRange;

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RaySegment {
    start: Distance,
    length: Distance,
}

impl RaySegment {
    pub fn new(start: Distance, length: Distance) -> Self {
        Self { start, length }
    }

    pub fn end(&self) -> Distance {
        self.start + self.length
    }

    pub fn contains(&self, distance: Distance) -> bool {
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
            Some(Self::new(
                rhs.start,
                Distance::new(end - back.start).unwrap(),
            ))
        } else {
            None
        }
    }
}

impl From<DisRange> for RaySegment {
    fn from(value: DisRange) -> Self {
        let start = match value.start_bound() {
            Bound::Included(v) | Bound::Excluded(v) => *v,
            Bound::Unbounded => Distance::zero(),
        };
        let end = match value.end_bound() {
            Bound::Included(v) | Bound::Excluded(v) => *v,
            Bound::Unbounded => Distance::infinity(),
        };
        Self::new(start, Distance::new(end - start).unwrap())
    }
}
