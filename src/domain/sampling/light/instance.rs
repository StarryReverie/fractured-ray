use rand::prelude::*;

use crate::domain::math::geometry::{AllTransformation, Transform, Transformation};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::shape::def::{Shape, ShapeId};
use crate::domain::shape::instance::Instance;

use super::{LightSample, LightSampling};

#[derive(Debug)]
pub struct InstanceLightSampler {
    id: ShapeId,
    instance: Instance,
    sampler: Option<Box<dyn LightSampling>>,
    inv_transformation: AllTransformation,
}

impl InstanceLightSampler {
    pub fn new(id: ShapeId, instance: Instance) -> Self {
        let inv_transformation = instance.transformation().clone().inverse();
        let sampler = instance.prototype().get_light_sampler(id);
        Self {
            id,
            instance,
            sampler,
            inv_transformation,
        }
    }
}

impl LightSampling for InstanceLightSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<&dyn Shape> {
        Some(&self.instance)
    }

    fn sample_light(
        &self,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        if let Some(sampler) = &self.sampler {
            let intersection = intersection.transform(&self.inv_transformation);
            sampler
                .sample_light(&intersection, rng)
                .map(|sample| sample.transform(self.instance.transformation()))
        } else {
            None
        }
    }

    fn pdf_light(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        if let Some(sampler) = &self.sampler {
            let intersection = intersection.transform(&self.inv_transformation);
            let ray_next = ray_next.transform(&self.inv_transformation);
            sampler.pdf_light(&intersection, &ray_next)
        } else {
            Val(0.0)
        }
    }
}
