use std::fmt::Debug;

use getset::Getters;
use rand::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::math::numeric::Val;
use crate::domain::math::transformation::{AtomTransformation, Transform};
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

    pub fn scale_pdf(self, multiplier: Val) -> Self {
        Self {
            photon: self.photon.scale_throughput(multiplier.recip()),
        }
    }
}

impl<T> Transform<T> for PhotonSample
where
    T: AtomTransformation,
    Ray: Transform<T>,
{
    fn transform(&self, transformation: &T) -> Self {
        Self::new(PhotonRay::new(
            self.photon.ray().transform(transformation),
            self.photon.throughput(),
        ))
    }
}
