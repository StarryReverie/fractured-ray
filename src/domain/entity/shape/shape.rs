use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};

use crate::domain::geometry::{Point, UnitVector, Val};
use crate::domain::ray::RayTrace;

pub trait Shape: Debug + Send + Sync + 'static {
    fn hit(&self, ray: &RayTrace, range: DisRange) -> Option<RayIntersection>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RayIntersection {
    distance: Val,
    position: Point,
    normal: UnitVector,
    side: SurfaceSide,
}

impl RayIntersection {
    pub fn new(distance: Val, position: Point, normal: UnitVector, side: SurfaceSide) -> Self {
        Self {
            distance,
            position,
            normal,
            side,
        }
    }

    pub fn distance(&self) -> Val {
        self.distance
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn normal(&self) -> UnitVector {
        self.normal
    }

    pub fn side(&self) -> SurfaceSide {
        self.side
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SurfaceSide {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisRange((Bound<Val>, Bound<Val>));

impl DisRange {
    pub fn positive() -> Self {
        Self((Bound::Excluded(Val(0.0)), Bound::Unbounded))
    }

    pub fn shrink_end(self, end: Val) -> Self {
        let end = match self.0.1 {
            b @ Bound::Included(o) if o < end => b,
            b @ Bound::Excluded(o) if o < end => b,
            _ => Bound::Excluded(end),
        };
        (self.0.0, end).into()
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
