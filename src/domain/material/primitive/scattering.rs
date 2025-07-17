use rand::prelude::*;
use rand_distr::Exp;
use snafu::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::Vector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val, WrappedVal};
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
        loop {
            let (x, y, z) = rng.random::<(WrappedVal, WrappedVal, WrappedVal)>();
            let (x, y, z) = (Val(x * 2.0 - 1.0), Val(y * 2.0 - 1.0), Val(z * 2.0 - 1.0));
            if let Ok(direction) = Vector::new(x, y, z).normalize() {
                return Ray::new(start, direction);
            }
        }
    }
}

impl Material for Scattering {
    fn material_kind(&self) -> MaterialKind {
        MaterialKind::Scattering
    }

    fn albedo(&self) -> Color {
        self.albedo
    }

    fn shade(
        &self,
        context: &Context<'_>,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        let mut rng = rand::rng();

        let ray = Ray::new(intersection.position(), ray.direction());
        let closet = context
            .scene()
            .find_intersection(&ray, DisRange::positive());
        let closet_distance = closet.as_ref().map_or(Val::INFINITY, |c| c.0.distance());

        let exp_distr = Exp::new(self.density.0).expect("self.density should be positive");
        let scatter_distance = Val(rng.sample(exp_distr));

        if scatter_distance < closet_distance {
            let start = ray.at(scatter_distance);
            let scattering_ray = self.generate_next_ray(start, &mut rng);
            let color = context
                .renderer()
                .trace(scattering_ray, DisRange::positive(), depth + 1);
            color * self.albedo
        } else if let Some((intersection, material)) = closet {
            let kind = material.material_kind();
            let side = intersection.side();

            if kind == MaterialKind::Scattering && side == SurfaceSide::Back {
                let boundary = intersection.position();
                let passthrough_ray = Ray::new(boundary, ray.direction());
                context
                    .renderer()
                    .trace(passthrough_ray, DisRange::positive(), depth)
            } else {
                context.renderer().trace(ray, DisRange::positive(), depth)
            }
        } else {
            unreachable!("closet should not be None otherwise 1st branch is executed")
        }
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
