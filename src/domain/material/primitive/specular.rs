use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialExt, MaterialKind};
use crate::domain::math::algebra::{Product, UnitVector, Vector};
use crate::domain::math::numeric::Val;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{CoefficientSample, CoefficientSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct Specular {
    color: Color,
}

impl Specular {
    pub fn new(albedo: Color) -> Self {
        Self { color: albedo }
    }

    fn calc_next_ray(&self, ray: &Ray, intersection: &RayIntersection) -> Ray {
        let normal = intersection.normal();
        let dir = ray.direction();
        Ray::new(
            intersection.position(),
            (dir - Val(2.0) * dir.dot(normal) * normal)
                .normalize()
                .expect("reflective ray's direction should not be zero vector"),
        )
    }
}

impl Material for Specular {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Specular
    }

    fn bsdf(
        &self,
        _dir_out: UnitVector,
        _intersection: &RayIntersection,
        _dir_in: UnitVector,
    ) -> Vector {
        unimplemented!("dirac function in refractive BSDF can't be represented")
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: Ray,
        intersection: RayIntersection,
    ) -> Contribution {
        self.shade_scattering(context, state, &ray, &intersection, false)
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

    fn as_dyn(&self) -> &dyn Material {
        self
    }
}

impl CoefficientSampling for Specular {
    fn sample_coefficient(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        _rng: &mut dyn RngCore,
    ) -> CoefficientSample {
        let direction = self.calc_next_ray(ray, intersection);
        let pdf = self.pdf_coefficient(ray, intersection, &direction);
        CoefficientSample::new(direction, self.color.to_vector(), pdf)
    }

    fn pdf_coefficient(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
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

        let specular = Specular::new(Color::WHITE);

        let ray_next = specular.calc_next_ray(&ray, &intersection);
        assert_eq!(
            ray_next.direction(),
            Vector::new(-sqrt3_2, Val(0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );
    }
}
