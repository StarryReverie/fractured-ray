use rand::prelude::*;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::geometry::AllTransformation;
use crate::domain::math::numeric::Val;
use crate::domain::shape::def::ShapeId;
use crate::domain::shape::instance::Instance;
use crate::domain::{color::Color, math::geometry::Transform};

use super::{PhotonSample, PhotonSampling};

#[derive(Debug)]
pub struct InstancePhotonSampler {
    sampler: Option<Box<dyn PhotonSampling>>,
    transformation: AllTransformation,
}

impl InstancePhotonSampler {
    pub fn new(id: ShapeId, instance: Instance, emissive: Emissive) -> Self {
        let sampler = instance.prototype().get_photon_sampler(id, emissive);
        let transformation = instance.transformation().clone();
        Self {
            sampler,
            transformation,
        }
    }
}

impl PhotonSampling for InstancePhotonSampler {
    fn radiance(&self) -> Color {
        self.sampler
            .as_ref()
            .map_or(Color::BLACK, |sampler| sampler.radiance())
    }

    fn area(&self) -> Val {
        self.sampler
            .as_ref()
            .map_or(Val(0.0), |sampler| sampler.area())
    }

    fn sample_photon(&self, rng: &mut dyn RngCore) -> Option<PhotonSample> {
        let sample = (self.sampler)
            .as_ref()
            .and_then(|sampler| sampler.sample_photon(rng))?;
        Some(sample.transform(&self.transformation))
    }
}
