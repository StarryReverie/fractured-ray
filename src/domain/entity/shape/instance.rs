use std::sync::Arc;

use crate::domain::geometry::{
    AllTransformation, Rotation, Transform, Transformation, Translation,
};
use crate::domain::ray::Ray;

use super::{BoundingBox, DisRange, RayIntersection, Shape, ShapeKind};

#[derive(Debug, Clone)]
pub struct Instance {
    prototype: Arc<dyn Shape>,
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
    fn shape_kind(&self) -> ShapeKind {
        ShapeKind::Instance
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        let inv_transformation = self.transformation.clone().inverse();
        let ray = ray.transform(&inv_transformation);
        let intersection = self.prototype.hit(&ray, range)?;
        Some(intersection.transform(&self.transformation))
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let bbox = self.prototype.bounding_box()?;
        Some(bbox.transform(&self.transformation))
    }
}
