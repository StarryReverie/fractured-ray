use std::fmt::Debug;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;

pub trait NoiseGenerator: Debug + Send + Sync {
    fn evaluate(&self, position: Point) -> Val;
}
