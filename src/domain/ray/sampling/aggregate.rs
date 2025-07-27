use std::collections::HashMap;

use rand::prelude::*;
use rand_distr::Uniform;
use rand_distr::weighted::WeightedIndex;

use crate::domain::entity::Bvh;
use crate::domain::material::def::Material;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val, WrappedVal};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeContainer, ShapeId};

use super::{LightSample, LightSampling, PointSample, PointSampling};

#[derive(Debug)]
pub struct AggregatePointSampler {
    samplers: Vec<Box<dyn PointSampling>>,
    weights: Vec<Val>,
    index_sampler: WeightedIndex<WrappedVal>,
}

impl AggregatePointSampler {
    pub fn new(samplers: Vec<(Box<dyn PointSampling>, Val)>) -> Self {
        let weights = samplers
            .iter()
            .map(|(_, Val(w))| w.max(0.0))
            .collect::<Vec<_>>();
        let index_sampler = WeightedIndex::new(weights).unwrap();

        let total = index_sampler.total_weight();
        let weights = index_sampler.weights().map(|w| Val(w / total)).collect();
        println!("{weights:?}");

        let samplers = samplers.into_iter().map(|(s, _)| s).collect();
        Self {
            samplers,
            weights,
            index_sampler,
        }
    }
}

impl PointSampling for AggregatePointSampler {
    fn id(&self) -> Option<ShapeId> {
        unreachable!("AggregatePointSampler::id() doesn't have a unique ID")
    }

    fn shape(&self) -> Option<&dyn Shape> {
        unreachable!("AggregatePointSampler doesn't have a unique inner shape")
    }

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample> {
        let which = self.index_sampler.sample(rng);
        (self.samplers.get(which))
            .and_then(|sampler| sampler.sample_point(rng))
            .map(|sample| sample.scale_pdf(self.weights[which]))
    }

    fn pdf_point(&self, point: Point, _checked_inside: bool) -> Val {
        for (i, sampler) in self.samplers.iter().enumerate() {
            let pdf = sampler.pdf_point(point, false);
            if pdf != Val(0.0) {
                return pdf * self.weights[i];
            }
        }
        Val(0.0)
    }
}

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
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let which = rng.sample(Uniform::new(0, self.ids.len()).unwrap());
        let id = self.ids[which];
        (self.lights.lights.get(&id))
            .and_then(|light| light.sample_light(ray, intersection, material, rng))
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

#[cfg(test)]
mod tests {
    use crate::domain::ray::sampling::TrianglePointSampler;
    use crate::domain::shape::def::ShapeKind;
    use crate::domain::shape::primitive::Triangle;

    use super::*;

    #[test]
    fn aggregate_point_sampler_pdf_point_succeeds() {
        let triangle1 = Triangle::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(-1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(1.0), Val(0.0)),
        )
        .unwrap();
        let area1 = triangle1.area();
        let sampler1: Box<dyn PointSampling> = Box::new(TrianglePointSampler::new(
            ShapeId::new(ShapeKind::Triangle, 0),
            triangle1,
        ));

        let triangle2 = Triangle::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(2.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(0.0)),
        )
        .unwrap();
        let area2 = triangle2.area();
        let sampler2: Box<dyn PointSampling> = Box::new(TrianglePointSampler::new(
            ShapeId::new(ShapeKind::Triangle, 1),
            triangle2,
        ));

        let samplers = vec![(sampler1, area1), (sampler2, area2)];
        let sampler = AggregatePointSampler::new(samplers);

        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.0), Val(0.0), Val(1.0)), false),
            Val(0.0)
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.0), Val(0.0), Val(0.0)), false),
            Val(0.4)
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(-0.5), Val(0.5), Val(0.0)), false),
            Val(0.4)
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.5), Val(0.5), Val(0.0)), false),
            Val(0.4)
        );
    }
}
