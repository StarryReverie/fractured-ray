use std::fmt::Debug;

use rand::prelude::*;

use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::shape::def::{Shape, ShapeId};

pub trait PointSampling: Debug + Send + Sync {
    fn id(&self) -> Option<ShapeId>;

    fn shape(&self) -> Option<&dyn Shape>;

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample>;

    fn pdf_point(&self, point: Point, checked_inside: bool) -> Val;
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn point(&self) -> Point {
        self.point
    }

    pub fn normal(&self) -> UnitVector {
        self.normal
    }

    pub fn pdf(&self) -> Val {
        self.pdf
    }

    pub fn shape_id(&self) -> ShapeId {
        self.shape_id
    }

    pub fn scale_pdf(self, multiplier: Val) -> Self {
        Self {
            pdf: self.pdf * multiplier,
            ..self
        }
    }
}
