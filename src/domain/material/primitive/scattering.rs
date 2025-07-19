use rand::prelude::*;
use rand_distr::Exp;
use snafu::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::sampling::{CoefSample, CoefSampling};
use crate::domain::ray::{Ray, RayIntersection, SurfaceSide};
use crate::domain::renderer::Context;

#[derive(Debug, Clone, PartialEq)]
pub struct Scattering {
    albedo: Color,
    density: Val,
}

impl Scattering {
    pub fn new(albedo: Color, density: Val) -> Result<Self, TryNewScatteringError> {
        ensure!(density > Val(0.0), InvalidDensitySnafu);
        Ok(Self { albedo, density })
    }

    fn generate_next_ray(&self, start: Point, rng: &mut dyn RngCore) -> Ray {
        let direction = UnitVector::random(rng);
        Ray::new(start, direction)
    }
}

impl Material for Scattering {
    fn material_kind(&self) -> MaterialKind {
        MaterialKind::Scattering
    }

    fn albedo(&self) -> Color {
        self.albedo
    }

    fn bsdf(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        todo!("BSSRDF sampling is not yet implemented")
    }

    fn shade(
        &self,
        context: &mut Context<'_>,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        let ray = Ray::new(intersection.position(), ray.direction());
        let closet = context
            .scene()
            .find_intersection(&ray, DisRange::positive());
        let closet_distance = closet.as_ref().map_or(Val::INFINITY, |c| c.0.distance());

        let exp_distr = Exp::new(self.density.0).expect("self.density should be positive");
        let scatter_distance = Val((*context.rng()).sample(exp_distr));

        if scatter_distance < closet_distance {
            let start = ray.at(scatter_distance);
            let scattering_ray = self.generate_next_ray(start, *context.rng());
            let color =
                context
                    .renderer()
                    .trace(context, scattering_ray, DisRange::positive(), depth + 1);
            color * self.albedo
        } else if let Some((intersection, id)) = closet {
            let entities = context.scene().get_entities();
            let material = entities.get_material(id.material_id()).unwrap();
            let kind = material.material_kind();
            let side = intersection.side();

            if kind == MaterialKind::Scattering && side == SurfaceSide::Back {
                let boundary = intersection.position();
                let passthrough_ray = Ray::new(boundary, ray.direction());
                context
                    .renderer()
                    .trace(context, passthrough_ray, DisRange::positive(), depth)
            } else {
                context
                    .renderer()
                    .trace(context, ray, DisRange::positive(), depth)
            }
        } else {
            unreachable!("closet should not be None otherwise 1st branch is executed")
        }
    }

    fn as_dyn(&self) -> &dyn Material {
        self
    }
}

impl CoefSampling for Scattering {
    fn coef_sample(
        &self,
        _ray: &Ray,
        _intersection: &RayIntersection,
        _rng: &mut dyn RngCore,
    ) -> CoefSample {
        todo!("BSSRDF sampling is not yet implemented")
    }

    fn coef_pdf(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        todo!("BSSRDF sampling is not yet implemented")
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewScatteringError {
    #[snafu(display("density is not positive"))]
    InvalidDensity,
}
