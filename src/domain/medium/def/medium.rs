use std::fmt::Debug;

use getset::CopyGetters;

use crate::domain::color::Spectrum;
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::ray::event::RayScattering;

pub trait Medium: Send + Sync {
    fn kind(&self) -> MediumKind;

    fn transmittance(&self, start: Point, length: Val) -> Spectrum;

    fn phase(
        &self,
        dir_out: UnitVector,
        scattering: &RayScattering,
        dir_in: UnitVector,
    ) -> Spectrum;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediumKind {
    Isotropic,
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
    fn add_medium<M: Medium>(&mut self, medium: M) -> MediumId
    where
        Self: Sized;

    fn get_medium(&self, id: MediumId) -> Option<&dyn Medium>;
}
