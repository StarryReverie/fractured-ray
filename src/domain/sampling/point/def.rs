use std::fmt::Debug;

use getset::CopyGetters;
use rand::prelude::*;

use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::{Sequential, Point, Transform};
use crate::domain::math::numeric::Val;
use crate::domain::shape::def::RefDynShape;
use crate::domain::shape::util::ShapeId;

pub trait PointSampling: Debug + Send + Sync {
    fn id(&self) -> Option<ShapeId>;

    fn shape(&self) -> Option<RefDynShape>;

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample>;

    fn pdf_point(&self, point: Point, checked_inside: bool) -> Val;
}

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PointSample {
    point: Point,
    normal: UnitVector,
    pdf: Val,
    shape_id: ShapeId,
}

impl PointSample {
    pub fn new(point: Point, normal: UnitVector, pdf: Val, shape_id: ShapeId) -> Self {
        Self {
            point,
            normal,
            pdf,
            shape_id,
        }
    }

    pub fn scale_pdf(self, multiplier: Val) -> Self {
        Self {
            pdf: self.pdf * multiplier,
            ..self
        }
    }
}

impl Transform<Sequential> for PointSample {
    fn transform(&self, transformation: &Sequential) -> Self {
        Self::new(
            self.point.transform(transformation),
            self.normal.transform(transformation),
            self.pdf,
            self.shape_id,
        )
    }
}
