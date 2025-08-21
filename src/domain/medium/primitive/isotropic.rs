use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::numeric::Val;
use crate::domain::medium::def::medium::{Medium, MediumKind};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayScattering, RaySegment};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Isotropic {
    sigma_s: Spectrum,
    sigma_t: Spectrum,
}

impl Isotropic {
    pub fn new(albedo: Albedo, mean_free_path: Spectrum) -> Result<Self, TryNewIsotropicError> {
        ensure!(mean_free_path.red() > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(mean_free_path.green() > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(mean_free_path.blue() > Val(0.0), InvalidMeanFreePathSnafu);

        let sigma_t = Spectrum::new(
            mean_free_path.red().recip(),
            mean_free_path.green().recip(),
            mean_free_path.blue().recip(),
        );
        let sigma_s = albedo * sigma_t;
        Ok(Self { sigma_s, sigma_t })
    }
}

impl Medium for Isotropic {
    fn kind(&self) -> MediumKind {
        MediumKind::Isotropic
    }

    fn transmittance(&self, _ray: &Ray, segment: &RaySegment) -> Spectrum {
        Spectrum::new(
            (-self.sigma_t.red() * segment.length()).exp(),
            (-self.sigma_t.green() * segment.length()).exp(),
            (-self.sigma_t.blue() * segment.length()).exp(),
        )
    }

    fn phase(
        &self,
        _dir_out: UnitVector,
        _scattering: &RayScattering,
        _dir_in: UnitVector,
    ) -> Spectrum {
        const PHASE: Spectrum = Spectrum::broadcast(Val(0.25 / Val::FRAC_1_PI.0));
        PHASE
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewIsotropicError {
    #[snafu(display("mean free path's each component should be positive"))]
    InvalidMeanFreePath,
}
