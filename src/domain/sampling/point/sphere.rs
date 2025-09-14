use rand::prelude::*;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::shape::def::Shape;
use crate::domain::shape::primitive::Sphere;
use crate::domain::shape::util::ShapeId;
use crate::domain::{math::algebra::UnitVector, shape::def::RefDynShape};

use super::{PointSample, PointSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct SpherePointSampler {
    id: ShapeId,
    sphere: Sphere,
    area_inv: Val,
}

impl SpherePointSampler {
    pub fn new(id: ShapeId, sphere: Sphere) -> Self {
        let area_inv = sphere.area().recip();
        Self {
            id,
            sphere,
            area_inv,
        }
    }
}

impl PointSampling for SpherePointSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<RefDynShape> {
        Some((&self.sphere).into())
    }

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample> {
        let dir = UnitVector::random(rng);
        let point = dir * self.sphere.radius() + self.sphere.center();
        Some(PointSample::new(
            point,
            dir,
            self.pdf_point(point, true),
            self.id,
        ))
    }

    fn pdf_point(&self, point: Point, checked_inside: bool) -> Val {
        if checked_inside {
            return self.area_inv;
        }
        let dis_squared = (point - self.sphere.center()).norm_squared();
        if dis_squared == self.sphere.radius().powi(2) {
            self.area_inv
        } else {
            Val(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::geometry::Point;
    use crate::domain::shape::def::ShapeKind;
    use crate::domain::shape::util::ShapeId;

    use super::*;

    #[test]
    fn sphere_point_sampler_pdf_point_succeeds() {
        let sampler = SpherePointSampler::new(
            ShapeId::new(ShapeKind::Sphere, 0),
            Sphere::new(Point::new(Val(0.0), Val(0.0), Val(0.0)), Val(0.5)).unwrap(),
        );

        assert_eq!(
            sampler.pdf_point(Point::new(Val(1.5), Val(0.0), Val(0.0)), false),
            Val(0.0),
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.5), Val(0.0), Val(0.0)), false),
            Val::FRAC_1_PI,
        );
    }
}
