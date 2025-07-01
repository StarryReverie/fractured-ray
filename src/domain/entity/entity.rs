use crate::domain::ray::RayTrace;

use super::shape::{DisRange, RayIntersection, Shape};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(usize);

impl Id {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct Entity {
    id: Id,
    shape: Box<dyn Shape>,
}

impl Entity {
    pub fn new<S: Shape>(id: Id, shape: S) -> Self {
        Entity {
            id,
            shape: Box::new(shape),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn hit(&self, ray: &RayTrace, range: DisRange) -> Option<RayIntersection> {
        self.shape.hit(ray, range)
    }
}
