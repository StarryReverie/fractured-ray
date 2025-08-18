use std::any::Any;
use std::fmt::Debug;

use crate::domain::color::Spectrum;
use crate::domain::math::algebra::UnitVector;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{BsdfSampling, BssrdfSampling};

pub trait Material: Debug + Send + Sync {
    fn kind(&self) -> MaterialKind;

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: Ray,
        intersection: RayIntersection,
    ) -> Contribution;

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    );

    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId {
    kind: MaterialKind,
    index: u32,
}

impl MaterialId {
    pub fn new(kind: MaterialKind, index: u32) -> Self {
        Self { kind, index }
    }

    pub fn kind(&self) -> MaterialKind {
        self.kind
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}

pub trait MaterialContainer: Debug + Send + Sync + 'static {
    fn add_material<M>(&mut self, material: M) -> MaterialId
    where
        Self: Sized,
        M: Material + Any;

    fn get_material(&self, id: MaterialId) -> Option<&dyn Material>;
}
