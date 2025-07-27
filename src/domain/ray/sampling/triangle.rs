use rand::prelude::*;

use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::shape::def::{Shape, ShapeId};
use crate::domain::shape::primitive::Triangle;

use super::{PointSample, PointSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct TrianglePointSampler {
    id: ShapeId,
    shape: Triangle,
    normal: UnitVector,
    area_inv: Val,
}

impl TrianglePointSampler {
    pub fn new(id: ShapeId, shape: Triangle) -> Self {
        let normal = shape.normal(shape.vertex0());
        let area_inv = shape.area().recip();
        Self {
            id,
            shape,
            normal,
            area_inv,
        }
    }
}

impl PointSampling for TrianglePointSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<&dyn Shape> {
        Some(&self.shape)
    }

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample> {
        let (mut r1, mut r2) = (Val(rng.random()), Val(rng.random()));
        if r1 + r2 > Val(1.0) {
            r1 = Val(1.0) - r1;
            r2 = Val(1.0) - r2;
        }
        let point = (Val(1.0) - r1 - r2) * self.shape.vertex0().into_vector()
            + r1 * self.shape.vertex1().into_vector()
            + r2 * self.shape.vertex2().into_vector();
        let point = Point::from(point);
        Some(PointSample::new(
            point,
            self.shape.normal(point),
            self.pdf_point(point, true),
            self.id,
        ))
    }

    fn pdf_point(&self, point: Point, checked_inside: bool) -> Val {
        if checked_inside {
            return self.area_inv;
        }
        let v0 = self.shape.vertex0();
        let v1 = self.shape.vertex1();
        let v2 = self.shape.vertex2();
        let normal = self.shape.normal(point);
        let inside = (point - v0).is_perpendicular_to(normal)
            && (v1 - v0).cross(point - v0).dot(normal) >= Val(0.0)
            && (v2 - v0).cross(point - v0).dot(normal) <= Val(0.0)
            && (v2 - v1).cross(point - v1).dot(normal) >= Val(0.0);
        if inside { self.area_inv } else { Val(0.0) }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::shape::def::ShapeKind;

    use super::*;

    #[test]
    fn triangle_point_sampler_pdf_point_succeeds() {
        let triangle = Triangle::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(-1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(1.0), Val(0.0)),
        )
        .unwrap();
        let sampler = TrianglePointSampler::new(ShapeId::new(ShapeKind::Triangle, 0), triangle);

        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.0), Val(0.0), Val(1.0)), false),
            Val(0.0)
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.0), Val(0.0), Val(0.0)), false),
            Val(2.0)
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(-0.5), Val(0.5), Val(0.0)), false),
            Val(2.0)
        );
    }
}
