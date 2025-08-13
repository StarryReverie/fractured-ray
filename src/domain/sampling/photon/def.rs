use std::fmt::Debug;

use getset::Getters;
use rand::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::math::geometry::{AllTransformation, Transform};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::photon::PhotonRay;

pub trait PhotonSampling: Debug + Send + Sync {
    fn radiance(&self) -> Spectrum;

    fn area(&self) -> Val;

    fn sample_photon(&self, rng: &mut dyn RngCore) -> Option<PhotonSample>;
}

#[derive(Debug, Clone, PartialEq, Getters)]
pub struct PhotonSample {
    #[getset(get = "pub")]
    photon: PhotonRay,
}

impl PhotonSample {
    pub fn new(photon: PhotonRay) -> Self {
        Self { photon }
    }

    pub fn into_photon(self) -> PhotonRay {
        self.photon
    }

    pub fn scale_pdf(self, multiplier: Val) -> Self {
        Self {
            photon: PhotonRay::new(
                Ray::new(self.photon.start(), self.photon.direction()),
                self.photon.throughput() / multiplier,
            ),
        }
    }
}

impl Transform<AllTransformation> for PhotonSample {
    fn transform(&self, transformation: &AllTransformation) -> Self {
        Self::new(PhotonRay::new(
            self.photon.ray().transform(transformation),
            self.photon.throughput(),
        ))
    }
}
