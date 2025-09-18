use std::fmt::Debug;

use getset::CopyGetters;
use rand::prelude::*;

use crate::domain::math::geometry::{Normal, Point};
use crate::domain::math::numeric::Val;
use crate::domain::math::transformation::{AtomTransformation, Transform};
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
    normal: Normal,
    pdf: Val,
    shape_id: ShapeId,
}

impl PointSample {
    pub fn new(point: Point, normal: Normal, pdf: Val, shape_id: ShapeId) -> Self {
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

impl<T> Transform<T> for PointSample
where
    T: AtomTransformation,
    Point: Transform<T>,
    Normal: Transform<T>,
{
    fn transform(&self, transformation: &T) -> Self {
        Self::new(
            self.point.transform(transformation),
            self.normal.transform(transformation),
            self.pdf,
            self.shape_id,
        )
    }
}
