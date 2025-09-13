use rand::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::math::numeric::Val;
use crate::domain::medium::def::{Medium, MediumKind};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RaySegment;
use crate::domain::renderer::{Contribution, RtContext, RtState};
use crate::domain::scene::volume::VolumeScene;

use super::MediumId;

pub struct AggregateMedium<'a> {
    volume_scene: &'a dyn VolumeScene,
    segments: &'a [(RaySegment, MediumId)],
}

impl<'a> AggregateMedium<'a> {
    pub fn new(volume_scene: &'a dyn VolumeScene, segments: &'a [(RaySegment, MediumId)]) -> Self {
        Self {
            volume_scene,
            segments,
        }
    }
}

impl<'a> Medium for AggregateMedium<'a> {
    fn kind(&self) -> MediumKind {
        unimplemented!("AggregateMedium doesn't have a unique kind")
    }

    fn transmittance(&self, ray: &Ray, segment: &RaySegment) -> Spectrum {
        (self.segments.iter()).fold(Spectrum::broadcast(Val(1.0)), |tr, (cur, id)| {
            let Some(seg_intersection) = segment.intersect(cur) else {
                return tr;
            };
            let Some(medium) = self.volume_scene.get_boundaries().get_medium(*id) else {
                return tr;
            };
            tr * medium.transmittance(ray, &seg_intersection)
        })
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        segment: &RaySegment,
    ) -> Contribution {
        if state.skip_medium_inscattering() {
            return Contribution::new();
        }
        if self.segments.is_empty() {
            return Contribution::new();
        }

        let which = context.rng().random_range(0..self.segments.len());
        let Some(segment) = segment.intersect(&self.segments[which].0) else {
            return Contribution::new();
        };
        let id = self.segments[which].1;
        let Some(medium) = self.volume_scene.get_boundaries().get_medium(id) else {
            return Contribution::new();
        };
        medium.shade(context, state, ray, &segment) * Val::from(self.segments.len()).recip()
    }
}
