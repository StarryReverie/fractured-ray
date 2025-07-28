use crate::domain::material::primitive::Emissive;
use crate::domain::shape::def::ShapeId;

use super::light::LightSampling;
use super::photon::PhotonSampling;

pub trait Sampleable {
    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>>;

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>>;
}
