use crate::domain::color::Spectrum;
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::ray::event::RayScattering;

pub trait Medium: Send + Sync {
    fn kind(&self) -> MediumKind;

    fn attenuation(&self) -> Spectrum;

    fn transmittance(&self, start: Point, end: Point) -> Spectrum {
        let d = self.attenuation() * (end - start).norm();
        Spectrum::new(-d.red().exp(), -d.green().exp(), -d.blue().exp())
    }

    fn phase(
        &self,
        dir_out: UnitVector,
        scattering: &RayScattering,
        dir_in: UnitVector,
    ) -> Spectrum;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediumKind {}
