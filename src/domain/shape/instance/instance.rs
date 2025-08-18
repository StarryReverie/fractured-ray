use std::sync::Arc;

use getset::Getters;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::{
    AllTransformation, Point, Rotation, Transform, Transformation, Translation,
};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::{InstanceLightSampler, LightSampling};
use crate::domain::sampling::photon::{InstancePhotonSampler, PhotonSampling};
use crate::domain::shape::def::{BoundingBox, Shape, ShapeId, ShapeKind};

#[derive(Debug, Clone, Getters)]
pub struct Instance {
    #[getset(get = "pub")]
    prototype: Arc<dyn Shape>,
    #[getset(get = "pub")]
    transformation: AllTransformation,
}

impl Instance {
    pub fn new(prototype: Arc<dyn Shape>, transformation: AllTransformation) -> Self {
        Self {
            prototype,
            transformation,
        }
    }

    pub fn of(prototype: Arc<dyn Shape>) -> Self {
        Self {
            prototype,
            transformation: AllTransformation::default(),
        }
    }

    pub fn wrap<S: Shape>(prototype: S) -> Self {
        Self::of(Arc::new(prototype))
    }

    pub fn rotate(self, rotation: Rotation) -> Self {
        Self {
            transformation: AllTransformation {
                rotation,
                ..self.transformation
            },
            ..self
        }
    }

    pub fn translate(self, translation: Translation) -> Self {
        Self {
            transformation: AllTransformation {
                translation,
                ..self.transformation
            },
            ..self
        }
    }
}

impl Shape for Instance {
    fn kind(&self) -> ShapeKind {
        ShapeKind::Instance
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        let inv_transformation = self.transformation.clone().inverse();
        let ray = ray.transform(&inv_transformation);
        let intersection = self.prototype.hit(&ray, range)?;
        Some(intersection.transform(&self.transformation))
    }

    fn area(&self) -> Val {
        self.prototype.area()
    }

    fn normal(&self, position: Point) -> UnitVector {
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
    use crate::domain::math::algebra::{UnitVector, Vector};
    use crate::domain::math::geometry::Point;
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
                UnitVector::x_direction(),
                UnitVector::z_direction(),
                Val::PI / Val(4.0),
            ))
            .translate(Translation::new(Vector::new(Val(0.0), Val(0.0), Val(-2.0))));

        let ray = Ray::new(
            Point::new(Val(0.0), Val(2.0).sqrt(), Val(-1.0)),
            UnitVector::z_direction(),
        );

        let intersection = instance.hit(&ray, DisRange::positive()).unwrap();

        assert_eq!(intersection.distance(), Val(1.0));
        assert_eq!(
            intersection.position(),
            Point::new(Val(0.0), Val(2.0).sqrt(), Val(0.0))
        );
        assert_eq!(intersection.normal(), -UnitVector::z_direction());
        assert_eq!(intersection.side(), SurfaceSide::Front);
    }
}
