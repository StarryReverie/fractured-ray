use std::fmt::Debug;

use enum_dispatch::enum_dispatch;

use crate::domain::color::Spectrum;
use crate::domain::material::primitive::*;
use crate::domain::math::algebra::UnitVector;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{BsdfSampling, BssrdfSampling};

use super::DynMaterial;

#[enum_dispatch]
pub trait Material: Debug + Send + Sync {
    fn kind(&self) -> MaterialKind;

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution;

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: &PhotonRay,
        intersection: &RayIntersection,
    );
}

pub trait BsdfMaterial: Material + BsdfSampling {
    fn bsdf(
        &self,
        dir_out: UnitVector,
        intersection: &RayIntersection,
        dir_in: UnitVector,
    ) -> Spectrum;
}

pub trait BssrdfMaterial: Material + BssrdfSampling {
    fn bssrdf_direction(&self, intersection_in: &RayIntersection, dir_in: UnitVector) -> Spectrum;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialKind {
    Blurry,
    Diffuse,
    Emissive,
    Glossy,
    Refractive,
    Scattering,
    Specular,
}
