use std::collections::HashMap;

use rand::{Rng, RngCore};
use rand_distr::Uniform;

use crate::domain::entity::Bvh;
use crate::domain::material::def::Material;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeContainer, ShapeId};

use super::{LightSample, LightSampling};

#[derive(Debug)]
pub struct MultiLightSampler {
    lights: LightContainer,
    ids: Vec<ShapeId>,
    bvh: Bvh<ShapeId>,
    weight: Val,
}

impl MultiLightSampler {
    pub fn new<I>(lights: I) -> Self
    where
        I: Iterator<Item = Box<dyn LightSampling>>,
    {
        let lights = LightContainer::new(lights);
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

impl LightSampling for MultiLightSampler {
    fn id(&self) -> Option<ShapeId> {
        unreachable!("MultiLightSampler::id() doesn't have a unique ID")
    }

    fn shape(&self) -> Option<&dyn Shape> {
        unreachable!("MultiLightSampler doesn't have a unique inner shape")
    }

    fn light_sample(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let which = rng.sample(Uniform::new(0, self.ids.len()).unwrap());
        let id = self.ids[which];
        (self.lights.lights.get(&id))
            .and_then(|light| light.light_sample(ray, intersection, material, rng))
            .map(|sample| sample.scale_pdf(self.weight))
    }

    fn light_pdf(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let res = (self.bvh).search(ray_next, DisRange::positive(), &self.lights);
        if let Some((_, id)) = res {
            let light = self.lights.lights.get(&id).unwrap();
            light.light_pdf(intersection, ray_next) * self.weight
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
    fn new<I>(lights: I) -> Self
    where
        I: Iterator<Item = Box<dyn LightSampling>>,
    {
        let lights = lights
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
