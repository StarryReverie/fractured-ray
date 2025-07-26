use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;

use crate::domain::material::def::Material;
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{Val, WrappedVal};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};
use crate::domain::shape::primitive::{Polygon, Triangle};

use super::{
    LightSample, LightSamplerAdapter, LightSampling, PointSample, PointSampling,
    TrianglePointSampler,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PolygonPointSampler {
    id: ShapeId,
    polygon: Polygon,
    triangles: Vec<TrianglePointSampler>,
    weights: Vec<Val>,
    normal: UnitVector,
    area_inv: Val,
    index_sampler: WeightedIndex<WrappedVal>,
}

impl PolygonPointSampler {
    pub fn new(id: ShapeId, polygon: Polygon) -> Self {
        let triangles = polygon.triangulate();

        let mut weights = triangles.iter().map(Triangle::area).collect::<Vec<_>>();
        let area_inv = weights.iter().cloned().sum::<Val>().recip();
        weights.iter_mut().for_each(|w| *w *= area_inv);
        let normal = triangles[0].normal();
        let index_sampler = WeightedIndex::new(weights.iter().map(|v| v.0)).unwrap();

        let triangles = (triangles.into_iter())
            .map(|triangle| TrianglePointSampler::new(id, triangle))
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

impl PointSampling for PolygonPointSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<&dyn Shape> {
        Some(&self.polygon)
    }

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample> {
        let which = self.index_sampler.sample(rng);
        (self.triangles.get(which))
            .and_then(|triangle| triangle.sample_point(rng))
            .map(|sample| sample.scale_pdf(self.weights[which]))
    }

    fn pdf_point(&self, point: Point) -> Val {
        for triangle in &self.triangles {
            if triangle.pdf_point(point) != Val(0.0) {
                return self.area_inv;
            }
        }
        Val(0.0)
    }

    fn pdf_point_checked_inside(&self, _point: Point) -> Val {
        self.area_inv
    }

    fn normal(&self, _point: Point) -> UnitVector {
        self.normal
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolygonLightSampler {
    inner: LightSamplerAdapter<PolygonPointSampler>,
}

impl PolygonLightSampler {
    pub fn new(id: ShapeId, polygon: Polygon) -> Self {
        let inner = PolygonPointSampler::new(id, polygon);
        let inner = LightSamplerAdapter::new(inner);
        Self { inner }
    }
}

impl LightSampling for PolygonLightSampler {
    fn id(&self) -> Option<ShapeId> {
        self.inner.id()
    }

    fn shape(&self) -> Option<&dyn Shape> {
        self.inner.shape()
    }

    fn sample_light(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        self.inner.sample_light(ray, intersection, material, rng)
    }

    fn pdf_light(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        self.inner.pdf_light(intersection, ray_next)
    }
}
