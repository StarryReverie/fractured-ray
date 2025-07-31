use std::ops::Bound;

use rand::prelude::*;

use crate::domain::math::algebra::Vector;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::photon::{Photon, PhotonMap, PhotonRay};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};

use super::Material;

pub trait MaterialExt: Material {
    fn shade_light(
        &self,
        context: &mut RtContext<'_>,
        ray: &Ray,
        intersection: &RayIntersection,
        mis: bool,
    ) -> Contribution {
        let scene = context.scene();
        let lights = scene.get_lights();
        let res = lights.sample_light(ray, intersection, self.as_dyn(), *context.rng());
        let Some(sample) = res else {
            return Contribution::new();
        };

        let ray_next = sample.ray_next();
        let range = (
            Bound::Excluded(Val(0.0)),
            Bound::Included(sample.distance()),
        );
        let res = scene.test_intersection(ray_next, range.into(), sample.shape_id());
        let (intersection_next, light_material) = if let Some(res) = res {
            let intersection_next = res.0;
            let material_id = res.1.material_id();
            let light_material = scene.get_entities().get_material(material_id).unwrap();
            (intersection_next, light_material)
        } else {
            return Contribution::new();
        };

        let pdf_light = sample.pdf();
        if pdf_light == Val(0.0) {
            return Contribution::new();
        }

        let weight = if mis {
            let pdf_scattering = self.pdf_coefficient(ray, intersection, ray_next);
            pdf_light / (pdf_light + pdf_scattering)
        } else {
            Val(1.0)
        };

        let coefficient = sample.coefficient();
        let ray_next = sample.into_ray_next();
        let radiance = light_material.shade(context, RtState::new(), ray_next, intersection_next);
        coefficient * radiance * weight
    }

    fn shade_scattering(
        &self,
        context: &mut RtContext<'_>,
        state_next: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
        mis: bool,
    ) -> Contribution {
        let renderer = context.renderer();

        let sample = self.sample_coefficient(ray, intersection, *context.rng());
        let ray_next = sample.ray_next();

        let pdf_scattering = sample.pdf();
        if pdf_scattering == Val(0.0) {
            return Contribution::new();
        }

        let weight = if mis {
            let lights = context.scene().get_lights();
            let pdf_light = lights.pdf_light(intersection, ray_next);
            pdf_scattering / (pdf_light + pdf_scattering)
        } else {
            Val(1.0)
        };

        let coefficient = sample.coefficient();
        let ray_next = sample.into_ray_next();
        let radiance = renderer.trace(context, state_next, ray_next, DisRange::positive());
        coefficient * radiance * weight
    }

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

    fn estimate_flux(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        photon_map: &PhotonMap,
        radius: Val,
    ) -> (Vector, Val) {
        let mut flux = Vector::zero();
        let photons = photon_map.search(intersection.position(), radius);
        let len = photons.len();
        for photon in photons {
            let bsdf = self.bsdf(-ray.direction(), intersection, photon.direction());
            flux = flux + bsdf * photon.throughput();
        }
        (flux, len.into())
    }
}

impl<M> MaterialExt for M where M: Material {}
