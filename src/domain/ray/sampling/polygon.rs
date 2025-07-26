use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;

use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{Val, WrappedVal};
use crate::domain::shape::def::{Shape, ShapeId};
use crate::domain::shape::primitive::Polygon;

use super::{PointSample, PointSampling, TrianglePointSampler};

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

        let area_inv = polygon.area().recip();
        let weights = (triangles.iter())
            .map(|triangle| triangle.area() * area_inv)
            .collect::<Vec<_>>();
        let normal = polygon.normal(triangles[0].vertex0());
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
}
