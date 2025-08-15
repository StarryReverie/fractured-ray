use std::any::Any;

use rand::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::material::def::{BsdfMaterial, BsdfMaterialExt, Material, MaterialKind};
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::numeric::Val;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{BsdfSample, BsdfSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct Specular {
    albedo: Albedo,
}

impl Specular {
    pub fn new(albedo: Albedo) -> Self {
        Self { albedo }
    }

    fn calc_next_ray(&self, ray: &Ray, intersection: &RayIntersection) -> Ray {
        let normal = intersection.normal();
        let dir = ray.direction();
        let direction = (dir - Val(2.0) * dir.dot(normal) * normal)
            .normalize()
            .expect("reflective ray's direction should not be zero vector");
        intersection.spawn(direction)
    }
}

impl Material for Specular {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Specular
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: Ray,
        intersection: RayIntersection,
    ) -> Contribution {
        let state_next = state.with_skip_emissive(false);
        self.shade_scattering(context, state_next, &ray, &intersection)
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        let state_next = state.with_has_specular(true);
        self.maybe_bounce_next_photon(context, state_next, photon, intersection);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl BsdfMaterial for Specular {
    fn bsdf(
        &self,
        _dir_out: UnitVector,
        _intersection: &RayIntersection,
        _dir_in: UnitVector,
    ) -> Spectrum {
        Spectrum::zero()
    }
}

impl BsdfSampling for Specular {
    fn sample_bsdf(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        _rng: &mut dyn RngCore,
    ) -> BsdfSample {
        let direction = self.calc_next_ray(ray, intersection);
        let pdf = self.pdf_bsdf(ray, intersection, &direction);
        BsdfSample::new(direction, self.albedo.into(), pdf)
    }

    fn pdf_bsdf(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        Val(1.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::{UnitVector, Vector};
    use crate::domain::math::geometry::Point;
    use crate::domain::ray::SurfaceSide;

    use super::*;

    #[test]
    fn specular_calc_next_ray_succeeds() {
        let sqrt3_2 = Val(3.0).sqrt() / Val(2.0);

        let ray = Ray::new(
            Point::new(sqrt3_2, Val(0.5), Val(0.0)),
            Vector::new(-sqrt3_2, Val(-0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );

        let intersection = RayIntersection::new(
            Val(1.0),
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            UnitVector::y_direction(),
            SurfaceSide::Back,
        );

        let specular = Specular::new(Albedo::WHITE);

        let ray_next = specular.calc_next_ray(&ray, &intersection);
        assert_eq!(
            ray_next.direction(),
            Vector::new(-sqrt3_2, Val(0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );
    }
}
