use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use getset::CopyGetters;

use crate::domain::color::Spectrum;
use crate::domain::math::algebra::UnitVector;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RaySegment;
use crate::domain::renderer::{Contribution, RtContext, RtState};
use crate::domain::sampling::phase::PhaseSampling;

use super::{DynMedium, RefDynMedium};

#[enum_dispatch(Medium)]
pub trait Medium: Send + Sync {
    fn kind(&self) -> MediumKind;

    fn transmittance(&self, ray: &Ray, segment: &RaySegment) -> Spectrum;

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        segment: &RaySegment,
    ) -> Contribution;
}

pub trait HomogeneousMedium: Medium + PhaseSampling {
    fn sigma_s(&self) -> Spectrum;

    fn phase(&self, dir_out: UnitVector, dir_in: UnitVector) -> Spectrum;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediumKind {
    HenyeyGreenstein,
    Isotropic,
    Vacuum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MediumId {
    kind: MediumKind,
    index: u32,
}

impl MediumId {
    pub fn new(kind: MediumKind, index: u32) -> Self {
        Self { kind, index }
    }
}

pub trait MediumContainer: Debug + Send + Sync + 'static {
    fn add_medium(&mut self, medium: DynMedium) -> MediumId;

    fn get_medium(&self, id: MediumId) -> Option<RefDynMedium>;
}
