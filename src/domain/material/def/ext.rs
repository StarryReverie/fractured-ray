use rand::prelude::*;

use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::RayIntersection;
use crate::domain::ray::photon::{Photon, PhotonRay};
use crate::domain::renderer::{PmContext, PmState};

use super::Material;

pub trait MaterialExt: Material {
    fn store_photon(
        &self,
        context: &mut PmContext<'_>,
        photon: &PhotonRay,
        intersection: &RayIntersection,
    ) {
        context.photons().push(Photon::new(
            intersection.position(),
            -photon.direction(),
            photon.throughput(),
        ));
    }

    fn maybe_bounce_next_photon(
        &self,
        context: &mut PmContext<'_>,
        state_next: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        let renderer = context.renderer();
        let mut throughput = photon.throughput();

        let continue_prob = (throughput.x())
            .max(throughput.y())
            .max(throughput.z())
            .clamp(Val(0.0), Val(1.0));
        if Val(context.rng().random()) < continue_prob {
            throughput = throughput / continue_prob;
        } else {
            return;
        }

        let sample = self.sample_coefficient(photon.ray(), &intersection, *context.rng());
        let throughput_next = sample.coefficient() * throughput;
        let photon_next = PhotonRay::new(sample.into_ray_next(), throughput_next);
        renderer.emit(context, state_next, photon_next, DisRange::positive());
    }
}

impl<M> MaterialExt for M where M: Material {}
