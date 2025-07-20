use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;

use crate::domain::material::def::Material;
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::numeric::{DisRange, Val, WrappedVal};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId, ShapeKind};
use crate::domain::shape::primitive::{Polygon, Triangle};

use super::{LightSample, LightSampling, TriangleSampler};

#[derive(Debug, Clone, PartialEq)]
pub struct PolygonSampler {
    id: ShapeId,
    polygon: Polygon,
    triangles: Vec<TriangleSampler>,
    weights: Vec<Val>,
    normal: UnitVector,
    area_inv: Val,
    index_sampler: WeightedIndex<WrappedVal>,
}

impl PolygonSampler {
    pub fn new(id: ShapeId, polygon: Polygon) -> Self {
        let triangles = polygon.triangulate();

        let mut weights = triangles.iter().map(Triangle::area).collect::<Vec<_>>();
        let area_inv = weights.iter().cloned().sum::<Val>().recip();
        weights.iter_mut().for_each(|w| *w = *w * area_inv);
        let normal = triangles[0].normal();
        let index_sampler = WeightedIndex::new(weights.iter().map(|v| v.0)).unwrap();

        let triangles = (triangles.into_iter())
            .map(|triangle| TriangleSampler::new(id, triangle))
            .collect::<Vec<_>>();

        Self {
            id,
            polygon,
            triangles,
            weights,
            normal,
            area_inv,
            index_sampler,
        }
    }
}

impl LightSampling for PolygonSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<&dyn Shape> {
        Some(&self.polygon)
    }

    fn light_sample(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let which = self.index_sampler.sample(rng);
        (self.triangles.get(which))
            .and_then(|triangle| triangle.light_sample(ray, intersection, material, rng))
            .map(|sample| sample.scale_pdf(self.weights[which]))
    }

    fn light_pdf(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        if let Some(intersection_next) = self.polygon.hit(ray_next, DisRange::positive()) {
            let cos = self.normal.dot(ray_next.direction()).abs();
            let point = intersection_next.position();
            let dis_squared = (point - intersection.position()).norm_squared();
            self.area_inv * dis_squared / cos
        } else {
            Val(0.0)
        }
    }
}
