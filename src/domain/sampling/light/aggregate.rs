use std::collections::HashMap;

use rand::prelude::*;
use rand_distr::Uniform;

use crate::domain::entity::Bvh;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeContainer, ShapeId};

use super::{LightSample, LightSampling};

#[derive(Debug)]
pub struct AggregateLightSampler {
    lights: LightContainer,
    ids: Vec<ShapeId>,
    bvh: Bvh<ShapeId>,
    weight: Val,
}

impl AggregateLightSampler {
    pub fn new(samplers: Vec<Box<dyn LightSampling>>) -> Self {
        let lights = LightContainer::new(samplers);
        let ids: Vec<_> = lights.lights.keys().cloned().collect();
        let bboxes = (lights.lights.iter())
            .filter_map(|(id, light)| {
                light.shape().map(|shape| {
                    let bbox = shape
                        .bounding_box()
                        .expect("unbounded shape should not have a light sampler");
                    (*id, bbox)
                })
            })
            .collect();
        let bvh = Bvh::new(bboxes);
        let weight = Val::from(ids.len()).recip();
        Self {
            lights,
            ids,
            bvh,
            weight,
        }
    }
}

impl LightSampling for AggregateLightSampler {
    fn id(&self) -> Option<ShapeId> {
        unreachable!("AggregateLightSampler::id() doesn't have a unique ID")
    }

    fn shape(&self) -> Option<&dyn Shape> {
        unreachable!("AggregateLightSampler doesn't have a unique inner shape")
    }

    fn sample_light(
        &self,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let which = rng.sample(Uniform::new(0, self.ids.len()).unwrap());
        let id = self.ids[which];
        (self.lights.lights.get(&id))
            .and_then(|light| light.sample_light(intersection, rng))
            .map(|sample| sample.scale_pdf(self.weight))
    }

    fn pdf_light(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let res = (self.bvh).search(ray_next, DisRange::positive(), &self.lights);
        if let Some((_, id)) = res {
            let light = self.lights.lights.get(&id).unwrap();
            light.pdf_light(intersection, ray_next) * self.weight
        } else {
            Val(0.0)
        }
    }
}

#[derive(Debug)]
struct LightContainer {
    lights: HashMap<ShapeId, Box<dyn LightSampling>>,
}

impl LightContainer {
    fn new(lights: Vec<Box<dyn LightSampling>>) -> Self {
        let lights = (lights.into_iter())
            .flat_map(|light| light.id().map(|id| (id, light)))
            .collect();
        Self { lights }
    }
}

impl ShapeContainer for LightContainer {
    fn add_shape<S: Shape>(&mut self, _shape: S) -> ShapeId
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn get_shape(&self, id: ShapeId) -> Option<&dyn Shape> {
        self.lights.get(&id).and_then(|l| l.shape())
    }
}
