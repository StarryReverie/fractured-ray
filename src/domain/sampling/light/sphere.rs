use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::algebra::{Product, UnitVector, Vector};
use crate::domain::math::geometry::{Rotation, Transform};
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};
use crate::domain::shape::primitive::Sphere;

use super::{LightSample, LightSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct SphereLightSampler {
    id: ShapeId,
    sphere: Sphere,
}

impl SphereLightSampler {
    pub fn new(id: ShapeId, sphere: Sphere) -> Self {
        Self { id, sphere }
    }
}

impl LightSampling for SphereLightSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<&dyn Shape> {
        Some(&self.sphere)
    }

    fn sample_light(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let radius2 = self.sphere.radius().powi(2);
        let to_center = self.sphere.center() - intersection.position();
        let cos_max_half_cone_angle = (Val(1.0) - radius2 / to_center.norm_squared()).sqrt();

        let r1_2pi = Val(rng.random()) * Val(2.0) * Val::PI;
        let r2 = Val(rng.random());
        let z = Val(1.0) + r2 * (cos_max_half_cone_angle - Val(1.0));
        let tmp = (Val(1.0) - z.powi(2)).sqrt();
        let x = r1_2pi.cos() * tmp;
        let y = r1_2pi.sin() * tmp;
        let local_at_sphere = Vector::new(x, y, z) * self.sphere.radius();

        let global_dir = -to_center.normalize().unwrap_or(UnitVector::z_direction());
        let tr = Rotation::new(UnitVector::z_direction(), global_dir, Val(0.0));
        let at_sphere = local_at_sphere.transform(&tr);
        let Ok(direction) = (to_center + at_sphere).normalize() else {
            return None;
        };
        let ray_next = Ray::new(intersection.position(), direction);

        let bsdf = material.bsdf(-ray.direction(), intersection, ray_next.direction());
        if bsdf.norm_squared() != Val(0.0) {
            let cos = direction.dot(intersection.normal());
            let solid_angle = Val(2.0) * Val::PI * (Val(1.0) - cos_max_half_cone_angle);
            let pdf = solid_angle.recip();
            let coefficient = bsdf * cos / pdf;
            let distance = (self.sphere.center() + at_sphere - intersection.position()).norm();
            Some(LightSample::new(
                ray_next,
                coefficient,
                pdf,
                distance,
                self.id,
            ))
        } else {
            None
        }
    }

    fn pdf_light(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let radius2 = self.sphere.radius().powi(2);
        let to_center = self.sphere.center() - intersection.position();
        let cos_max_half_cone_angle = (Val(1.0) - radius2 / to_center.norm_squared()).sqrt();

        let cos_ray_center = ray_next.direction().dot(to_center.normalize().unwrap());
        if cos_ray_center >= cos_max_half_cone_angle {
            let solid_angle = Val(2.0) * Val::PI * (Val(1.0) - cos_max_half_cone_angle);
            solid_angle.recip()
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
    fn sphere_light_sampler_pdf_light_succeeds() {
        let sampler = SphereLightSampler::new(
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
            sampler.pdf_light(&intersection, &ray_next),
            Val(1.187948667)
        );
    }
}
