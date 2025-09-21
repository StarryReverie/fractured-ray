use std::sync::Arc;

use crate::domain::math::transformation::{Rotation, Scaling, Sequential, Translation};
use crate::domain::shape::mesh::MeshConstructor;
use crate::domain::shape::util::{ShapeConstructor, ShapeContainer, ShapeId};

#[derive(Debug, Clone)]
pub struct MeshInstanceConstructor {
    prototype: Arc<MeshConstructor>,
    transformation: Sequential,
}

impl MeshInstanceConstructor {
    pub fn new(prototype: Arc<MeshConstructor>, transformation: Sequential) -> Self {
        Self {
            prototype,
            transformation,
        }
    }

    pub fn of(prototype: Arc<MeshConstructor>) -> Self {
        Self {
            prototype,
            transformation: Sequential::default(),
        }
    }

    pub fn wrap(portotype: MeshConstructor) -> Self {
        Self::of(Arc::new(portotype))
    }

    pub fn scale(self, scaling: Scaling) -> Self {
        Self {
            transformation: self.transformation.with_scaling(scaling),
            ..self
        }
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

impl ShapeConstructor for MeshInstanceConstructor {
    fn construct(self: Box<Self>, container: &mut dyn ShapeContainer) -> Vec<ShapeId> {
        let transformation = Some(self.transformation);

        let prototype = Arc::unwrap_or_clone(self.prototype);
        let (triangles, polygons) = prototype.construct_impl(transformation);

        let mut ids = Vec::with_capacity(triangles.len() + polygons.len());
        for triangle in triangles {
            ids.push(container.add_shape(triangle.into()));
        }
        for polygon in polygons {
            ids.push(container.add_shape(polygon.into()));
        }
        ids
    }
}
