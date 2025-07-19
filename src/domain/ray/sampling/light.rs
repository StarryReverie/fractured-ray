use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::algebra::{Product, UnitVector, Vector};
use crate::domain::math::geometry::{Rotation, Transform};
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::ShapeId;
use crate::domain::shape::primitive::Sphere;

use super::{LightSample, LightSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct EmptySampler {}

impl EmptySampler {
    pub fn new() -> Self {
        Self {}
    }
}

impl LightSampling for EmptySampler {
    fn light_sample(
        &self,
        _ray: &Ray,
        _intersection: &RayIntersection,
        _material: &dyn Material,
        _rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        None
    }

    fn light_pdf(&self, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        Val(0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SphereSampler {
    id: ShapeId,
    shape: Sphere,
}

impl SphereSampler {
    pub fn new(id: ShapeId, shape: Sphere) -> Self {
        Self { id, shape }
    }
}

impl LightSampling for SphereSampler {
    fn light_sample(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let radius2 = self.shape.radius().powi(2);
        let to_center = self.shape.center() - intersection.position();
        let cos_max_half_cone_angle = (Val(1.0) - radius2 / to_center.norm_squared()).sqrt();

        let r1_2pi = Val(rng.random()) * Val(2.0) * Val::PI;
        let r2 = Val(rng.random());
        let z = Val(1.0) + r2 * (cos_max_half_cone_angle - Val(1.0));
        let tmp = (Val(1.0) - z.powi(2)).sqrt();
        let x = r1_2pi.cos() * tmp;
        let y = r1_2pi.sin() * tmp;
        let local_at_sphere = Vector::new(x, y, z) * self.shape.radius();

        let global_dir = -to_center.normalize().unwrap();
        let tr = Rotation::new(UnitVector::z_direction(), global_dir, Val(0.0));
        let at_sphere = local_at_sphere.transform(&tr);
        let direction = (to_center + at_sphere).normalize().unwrap();
        let ray_next = Ray::new(intersection.position(), direction);

        let bsdf = material.bsdf(ray, intersection, &ray_next);
        if bsdf != Val(0.0) {
            let cos = direction.dot(intersection.normal());
            let solid_angle = Val(2.0) * Val::PI * (Val(1.0) - cos_max_half_cone_angle);
            let pdf = solid_angle.recip();
            let coefficient = bsdf * cos / pdf;
            Some(LightSample::new(ray_next, coefficient, pdf, self.id))
        } else {
            None
        }
    }

    fn light_pdf(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let radius2 = self.shape.radius().powi(2);
        let to_center = self.shape.center() - intersection.position();
        let cos_max_half_cone_angle = (Val(1.0) - radius2 / to_center.norm_squared()).sqrt();

        let cos_ray_center = ray_next.direction().dot(to_center.normalize().unwrap());
        if cos_ray_center >= cos_max_half_cone_angle {
            let solid_angle = Val(2.0) * Val::PI * (Val(1.0) - cos_max_half_cone_angle);
            let pdf = solid_angle.recip();
            pdf
        } else {
            Val(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::geometry::Point;
    use crate::domain::ray::SurfaceSide;
    use crate::domain::shape::def::{ShapeId, ShapeKind};

    use super::*;

    #[test]
    fn sphere_sampler_light_pdf_succeeds() {
        let sampler = SphereSampler::new(
            ShapeId::new(ShapeKind::Sphere, 0),
            Sphere::new(Point::new(Val(0.0), Val(0.0), Val(0.0)), Val(2.0)).unwrap(),
        );

        let intersection = RayIntersection::new(
            Val(1.0),
            Point::new(Val(4.0), Val(0.0), Val(0.0)),
            -UnitVector::x_direction(),
            SurfaceSide::Front,
        );

        let ray_next = Ray::new(
            Point::new(Val(4.0), Val(0.0), Val(0.0)),
            Vector::new(Val(-3.0), Val(1.7320508676), Val(0.0))
                .normalize()
                .unwrap(),
        );

        assert_eq!(
            sampler.light_pdf(&intersection, &ray_next),
            Val(1.187948667)
        );
    }
}
