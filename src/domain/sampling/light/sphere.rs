use rand::prelude::*;

use crate::domain::math::algebra::{Product, UnitVector, Vector};
use crate::domain::math::geometry::{Frame, Point};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RayScattering};
use crate::domain::sampling::point::PointSample;
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

    fn sample_light_impl(
        &self,
        position: Point,
        ray_spawner: impl Fn(UnitVector) -> Ray,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let radius2 = self.sphere.radius().powi(2);

        let to_center = self.sphere.center() - position;
        let cos_max_spread = (Val(1.0) - radius2 / to_center.norm_squared()).sqrt();

        let r1_2pi = Val(rng.random()) * Val(2.0) * Val::PI;
        let r2 = Val(rng.random());
        let z = Val(1.0) + r2 * (cos_max_spread - Val(1.0));
        let tmp = (Val(1.0) - z.powi(2)).sqrt();
        let x = r1_2pi.cos() * tmp;
        let y = r1_2pi.sin() * tmp;
        let local_at_sphere = Vector::new(x, y, z) * self.sphere.radius();

        let global_dir = -to_center.normalize().unwrap_or(UnitVector::z_direction());
        let frame = Frame::new(global_dir);
        let at_sphere = frame.to_canonical(local_at_sphere);
        let Ok(direction) = (to_center + at_sphere).normalize() else {
            return None;
        };
        let ray_next = ray_spawner(direction);

        let solid_angle = Val(2.0) * Val::PI * (Val(1.0) - cos_max_spread);
        let pdf = solid_angle.recip();
        let distance = (self.sphere.center() + at_sphere - position).norm();
        Some(LightSample::new(ray_next, pdf, distance, self.id))
    }

    fn pdf_light_impl(&self, ray_next: &Ray, position: Point) -> Val {
        let radius2 = self.sphere.radius().powi(2);
        let to_center = self.sphere.center() - position;
        let cos_max_spread = (Val(1.0) - radius2 / to_center.norm_squared()).sqrt();

        let cos_ray_center = ray_next.direction().dot(to_center.normalize().unwrap());
        if cos_ray_center >= cos_max_spread {
            let solid_angle = Val(2.0) * Val::PI * (Val(1.0) - cos_max_spread);
            solid_angle.recip()
        } else {
            Val(0.0)
        }
    }
}

impl LightSampling for SphereLightSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<&dyn Shape> {
        Some(&self.sphere)
    }

    fn sample_light_surface(
        &self,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let ray_spawner = |dir| intersection.spawn(dir);
        self.sample_light_impl(intersection.position(), ray_spawner, rng)
    }

    fn pdf_light_surface(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        self.pdf_light_impl(ray_next, intersection.position())
    }

    fn sample_light_volume(
        &self,
        scattering: &RayScattering,
        preselected_light: Option<&PointSample>,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        if let Some(sample) = preselected_light {
            if self.id().is_none_or(|id| id != sample.shape_id()) {
                return None;
            }

            let Ok(dir_next) = (sample.point() - scattering.position()).normalize() else {
                return None;
            };
            let ray_next = scattering.spawn(dir_next);

            let radius2 = self.sphere.radius().powi(2);
            let to_center = self.sphere.center() - sample.point();
            let cos_max_spread = (Val(1.0) - radius2 / to_center.norm_squared()).sqrt();

            let cos_ray_center = ray_next.direction().dot(to_center.normalize().unwrap());
            if cos_ray_center >= cos_max_spread {
                let solid_angle = Val(2.0) * Val::PI * (Val(1.0) - cos_max_spread);
                let cond_pdf = solid_angle.recip() / sample.pdf();

                let distance = (sample.point() - scattering.position()).norm();
                Some(LightSample::new(
                    ray_next,
                    cond_pdf,
                    distance,
                    sample.shape_id(),
                ))
            } else {
                None
            }
        } else {
            let ray_spawner = |dir| scattering.spawn(dir);
            self.sample_light_impl(scattering.position(), ray_spawner, rng)
        }
    }

    fn pdf_light_volume(
        &self,
        scattering: &RayScattering,
        ray_next: &Ray,
        preselected_light: Option<&PointSample>,
    ) -> Val {
        if let Some(sample) = preselected_light {
            if self.has_nonzero_prob_given_preselected_light(ray_next, sample) {
                self.pdf_light_impl(ray_next, scattering.position()) / sample.pdf()
            } else {
                Val(0.0)
            }
        } else {
            self.pdf_light_impl(ray_next, scattering.position())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::geometry::Point;
    use crate::domain::ray::event::SurfaceSide;
    use crate::domain::shape::def::{ShapeId, ShapeKind};

    use super::*;

    #[test]
    fn sphere_light_sampler_pdf_light_surface_succeeds() {
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
            sampler.pdf_light_surface(&intersection, &ray_next),
            Val(1.187948667)
        );
    }

    #[test]
    fn sphere_light_sampler_pdf_light_volume_succeeds_when_ray_is_towards_preselected_light() {
        let shape_id = ShapeId::new(ShapeKind::Sphere, 0);
        let sampler = SphereLightSampler::new(
            shape_id,
            Sphere::new(Point::new(Val(0.0), Val(0.0), Val(0.0)), Val(2.0)).unwrap(),
        );

        let scattering = RayScattering::new(Val(1.0), Point::new(Val(4.0), Val(0.0), Val(0.0)));

        let light_point = Point::new(Val(1.0), Val(1.7320508676), Val(0.0));
        let light_point_pdf = sampler.sphere.area().recip();
        let preselected_light = PointSample::new(
            light_point,
            sampler.sphere.normal(light_point),
            light_point_pdf,
            shape_id,
        );

        let direction_next = (light_point - scattering.position()).normalize().unwrap();
        let ray_next = scattering.spawn(direction_next);

        assert_eq!(
            sampler.pdf_light_volume(&scattering, &ray_next, Some(&preselected_light)),
            Val(59.71281292110202),
        );
    }

    #[test]
    fn sphere_light_sampler_pdf_light_volume_succeeds_when_ray_is_not_towards_preselected_light() {
        let shape_id = ShapeId::new(ShapeKind::Sphere, 0);
        let sampler = SphereLightSampler::new(
            shape_id,
            Sphere::new(Point::new(Val(0.0), Val(0.0), Val(0.0)), Val(2.0)).unwrap(),
        );

        let scattering = RayScattering::new(Val(1.0), Point::new(Val(4.0), Val(0.0), Val(0.0)));

        let light_point = Point::new(Val(1.0), Val(1.7320508676), Val(0.0));
        let light_point_pdf = sampler.sphere.area().recip();
        let preselected_light = PointSample::new(
            light_point,
            sampler.sphere.normal(light_point),
            light_point_pdf,
            shape_id,
        );

        let ray_next = scattering.spawn(-UnitVector::x_direction());

        assert_eq!(
            sampler.pdf_light_volume(&scattering, &ray_next, Some(&preselected_light)),
            Val(0.0),
        );
    }
}
