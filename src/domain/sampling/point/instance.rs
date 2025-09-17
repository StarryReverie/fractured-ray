use rand::prelude::*;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::math::transformation::{Sequential, Transform, Transformation};
use crate::domain::sampling::Sampleable;
use crate::domain::shape::def::RefDynShape;
use crate::domain::shape::util::{Instance, ShapeId};

use super::{PointSample, PointSampling};

#[derive(Debug)]
pub struct InstancePointSampler {
    id: ShapeId,
    instance: Instance,
    sampler: Option<Box<dyn PointSampling>>,
    inv_transformation: Sequential,
}

impl InstancePointSampler {
    pub fn new(id: ShapeId, instance: Instance) -> Self {
        let inv_transformation = instance.transformation().clone().inverse();
        let sampler = instance.get_point_sampler(id);
        Self {
            id,
            instance,
            sampler,
            inv_transformation,
        }
    }
}

impl PointSampling for InstancePointSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<RefDynShape> {
        Some((&self.instance).into())
    }

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample> {
        (self.sampler.as_ref())
            .and_then(|sampler| sampler.sample_point(rng))
            .map(|sample| sample.transform(self.instance.transformation()))
    }

    fn pdf_point(&self, point: Point, checked_inside: bool) -> Val {
        self.sampler.as_ref().map_or(Val(0.0), |sampler| {
            sampler.pdf_point(point.transform(&self.inv_transformation), checked_inside)
        })
    }
}
