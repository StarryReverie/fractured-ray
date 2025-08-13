use std::any::Any;

use rand::prelude::*;
use rand_distr::Exp;
use snafu::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::{Ray, RayIntersection, SurfaceSide};
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};

#[derive(Debug, Clone, PartialEq)]
pub struct Scattering {
    color: Color,
    density: Val,
}

impl Scattering {
    pub fn new(color: Color, density: Val) -> Result<Self, TryNewScatteringError> {
        ensure!(density > Val(0.0), InvalidDensitySnafu);
        Ok(Self { color, density })
    }

    fn generate_next_ray(&self, start: Point, rng: &mut dyn RngCore) -> Ray {
        let direction = UnitVector::random(rng);
        Ray::new(start, direction)
    }
}

impl Material for Scattering {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Scattering
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: Ray,
        intersection: RayIntersection,
    ) -> Contribution {
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
                    .trace(context, state, scattering_ray, DisRange::positive());
            color * self.color.to_vector()
        } else if let Some((intersection, id)) = closet {
            let entities = context.scene().get_entities();
            let material = entities.get_material(id.material_id()).unwrap();
            let kind = material.kind();
            let side = intersection.side();

            if kind == MaterialKind::Scattering && side == SurfaceSide::Back {
                let boundary = intersection.position();
                let passthrough_ray = Ray::new(boundary, ray.direction());
                context
                    .renderer()
                    .trace(context, state, passthrough_ray, DisRange::positive())
            } else {
                context
                    .renderer()
                    .trace(context, state, ray, DisRange::positive())
            }
        } else {
            unreachable!("closet should not be None otherwise 1st branch is executed")
        }
    }

    fn receive(
        &self,
        _context: &mut PmContext<'_>,
        _state: PmState,
        _photon: PhotonRay,
        _intersection: RayIntersection,
    ) {
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewScatteringError {
    #[snafu(display("density is not positive"))]
    InvalidDensity,
}
