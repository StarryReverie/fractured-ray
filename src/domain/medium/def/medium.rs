use std::fmt::Debug;

use enum_dispatch::enum_dispatch;

use crate::domain::color::core::Spectrum;
use crate::domain::math::geometry::Direction;
use crate::domain::medium::primitive::*;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RaySegment;
use crate::domain::renderer::{Contribution, RtContext, RtState};
use crate::domain::sampling::phase::PhaseSampling;

use super::DynMedium;

#[enum_dispatch]
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

    fn phase(&self, dir_out: Direction, dir_in: Direction) -> Spectrum;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediumKind {
    HenyeyGreenstein,
    Isotropic,
    Vacuum,
}
