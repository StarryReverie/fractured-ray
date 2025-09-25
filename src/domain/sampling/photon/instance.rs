use rand::prelude::*;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::geometry::Area;
use crate::domain::math::transformation::{Sequential, Transform};
use crate::domain::sampling::Sampleable;
use crate::domain::shape::util::{Instance, ShapeId};

use super::{PhotonSample, PhotonSampling};

#[derive(Debug)]
pub struct InstancePhotonSampler {
    sampler: Option<Box<dyn PhotonSampling>>,
    transformation: Sequential,
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
    fn area(&self) -> Area {
        self.sampler
            .as_ref()
            .map_or(Area::zero(), |sampler| sampler.area())
    }

    fn sample_photon(&self, rng: &mut dyn RngCore) -> Option<PhotonSample> {
        let sample = (self.sampler)
            .as_ref()
            .and_then(|sampler| sampler.sample_photon(rng))?;
        Some(sample.transform(&self.transformation))
    }
}
