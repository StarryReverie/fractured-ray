use std::sync::Arc;

use crate::domain::math::geometry::{AllTransformation, Rotation, Transformation, Translation};
use crate::domain::shape::mesh::MeshConstructor;
use crate::domain::shape::util::{ShapeConstructor, ShapeContainer, ShapeId};

#[derive(Debug, Clone)]
pub struct MeshInstanceConstructor {
    prototype: Arc<MeshConstructor>,
    transformation: AllTransformation,
}

impl MeshInstanceConstructor {
    pub fn new(prototype: Arc<MeshConstructor>, transformation: AllTransformation) -> Self {
        Self {
            prototype,
            transformation,
        }
    }

    pub fn of(prototype: Arc<MeshConstructor>) -> Self {
        Self {
            prototype,
            transformation: AllTransformation::default(),
        }
    }

    pub fn wrap(portotype: MeshConstructor) -> Self {
        Self::of(Arc::new(portotype))
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

impl ShapeConstructor for MeshInstanceConstructor {
    fn construct<C: ShapeContainer>(self, container: &mut C) -> Vec<ShapeId> {
        let inv_transformation = Some(self.transformation.clone().inverse());
        let transformation = Some(self.transformation);

        let prototype = Arc::unwrap_or_clone(self.prototype);
        let (triangles, polygons) = prototype.construct_impl(transformation, inv_transformation);

        let mut ids = Vec::with_capacity(triangles.len() + polygons.len());
        for triangle in triangles {
            ids.push(container.add_shape(triangle));
        }
        for polygon in polygons {
            ids.push(container.add_shape(polygon));
        }
        ids
    }
}
