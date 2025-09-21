use std::ops::RangeBounds;
use std::sync::Arc;

use getset::Getters;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::geometry::{Area, Normal, Point};
use crate::domain::math::numeric::DisRange;
use crate::domain::math::transformation::*;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RayIntersectionPart};
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::{InstanceLightSampler, LightSampling};
use crate::domain::sampling::photon::{InstancePhotonSampler, PhotonSampling};
use crate::domain::sampling::point::{InstancePointSampler, PointSampling};
use crate::domain::shape::def::{BoundingBox, DynShape, Shape, ShapeKind};
use crate::domain::shape::util::ShapeId;

#[derive(Debug, Clone, Getters, PartialEq, Eq)]
pub struct Instance {
    #[getset(get = "pub")]
    prototype: Arc<DynShape>,
    #[getset(get = "pub")]
    transformation: Sequential,
}

impl Instance {
    pub fn new(prototype: Arc<DynShape>, transformation: Sequential) -> Self {
        Self {
            prototype,
            transformation,
        }
    }

    pub fn of(prototype: Arc<DynShape>) -> Self {
        Self {
            prototype,
            transformation: Sequential::default(),
        }
    }

    pub fn wrap<S: Into<DynShape>>(prototype: S) -> Self {
        Self::of(Arc::new(prototype.into()))
    }

    pub fn rotate(self, rotation: Rotation) -> Self {
        Self {
            transformation: self.transformation.with_rotation(rotation),
            ..self
        }
    }

    pub fn translate(self, translation: Translation) -> Self {
        Self {
            transformation: self.transformation.with_translation(translation),
            ..self
        }
    }
}

impl Shape for Instance {
    fn kind(&self) -> ShapeKind {
        ShapeKind::Instance
    }

    fn hit_part<'a>(&self, ray: &'a Ray, range: DisRange) -> Option<RayIntersectionPart<'a>> {
        let inv_tr = self.transformation.clone().inverse();

        let ray_tr = ray.clone().transform(&inv_tr);
        let range_tr = DisRange::from((
            range.start_bound().map(|d| d.transform(&inv_tr)),
            range.end_bound().map(|d| d.transform(&inv_tr)),
        ));

        let part_tr = self.prototype.hit_part(&ray_tr, range_tr)?;
        Some(RayIntersectionPart::new(
            part_tr.distance().transform(&self.transformation),
            ray,
        ))
    }

    fn complete_part(&self, part: RayIntersectionPart) -> RayIntersection {
        let inv_tr = self.transformation.clone().inverse();

        let distance_tr = part.distance().transform(&inv_tr);
        let ray_tr = part.ray().clone().transform(&inv_tr);
        let part_tr = RayIntersectionPart::new(distance_tr, &ray_tr);

        let intersection_tr = self.prototype.complete_part(part_tr);
        intersection_tr.transform(&self.transformation)
    }

    fn area(&self) -> Area {
        self.prototype.area().transform(&self.transformation)
    }

    fn normal(&self, position: Point) -> Normal {
        self.prototype
            .normal(position)
            .transform(&self.transformation)
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let bbox = self.prototype.bounding_box()?;
        Some(bbox.transform(&self.transformation))
    }
}

impl Sampleable for Instance {
    fn get_point_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        Some(Box::new(InstancePointSampler::new(shape_id, self.clone())))
    }

    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        Some(Box::new(InstanceLightSampler::new(shape_id, self.clone())))
    }

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        Some(Box::new(InstancePhotonSampler::new(
            shape_id,
            self.clone(),
            emissive,
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::Vector;
    use crate::domain::math::geometry::{Direction, Distance, Point};
    use crate::domain::math::numeric::Val;
    use crate::domain::ray::event::SurfaceSide;
    use crate::domain::shape::primitive::Polygon;

    use super::*;

    #[test]
    fn instance_hit_succeeds() {
        let prototype = Polygon::new([
            Point::new(Val(2.0), Val(1.0), Val(1.0)),
            Point::new(Val(2.0), Val(1.0), Val(-1.0)),
            Point::new(Val(2.0), Val(-1.0), Val(-1.0)),
            Point::new(Val(2.0), Val(-1.0), Val(1.0)),
        ])
        .unwrap();

        let instance = Instance::wrap(prototype)
            .rotate(Rotation::new(
                Direction::x_direction(),
                Direction::z_direction(),
                Val::PI / Val(4.0),
            ))
            .translate(Translation::new(Vector::new(Val(0.0), Val(0.0), Val(-2.0))));

        let ray = Ray::new(
            Point::new(Val(0.0), Val(2.0).sqrt(), Val(-1.0)),
            Direction::z_direction(),
        );

        let intersection = instance.hit(&ray, DisRange::positive()).unwrap();

        assert_eq!(intersection.distance(), Distance::new(Val(1.0)).unwrap());
        assert_eq!(
            intersection.position(),
            Point::new(Val(0.0), Val(2.0).sqrt(), Val(0.0))
        );
        assert_eq!(intersection.normal(), -Normal::z_direction());
        assert_eq!(intersection.side(), SurfaceSide::Front);
    }
}
