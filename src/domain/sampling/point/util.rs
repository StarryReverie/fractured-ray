use rand::prelude::*;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::shape::def::{Shape, ShapeId};

use super::{PointSample, PointSampling};

#[derive(Debug, Clone)]
pub struct EmptyPointSampler {}

impl EmptyPointSampler {
    pub fn new() -> Self {
        Self {}
    }
}

impl PointSampling for EmptyPointSampler {
    fn id(&self) -> Option<ShapeId> {
        None
    }

    fn shape(&self) -> Option<&dyn Shape> {
        None
    }

    fn sample_point(&self, _rng: &mut dyn RngCore) -> Option<PointSample> {
        None
    }

    fn pdf_point(&self, _point: Point, _checked_inside: bool) -> Val {
        Val(0.0)
    }
}
