use crate::domain::shape::def::ShapeId;

use super::light::LightSampling;

pub trait Sampleable {
    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>>;
}
