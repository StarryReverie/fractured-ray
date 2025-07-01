use crate::domain::ray::RayTrace;

use super::Entity;
use super::entity::Id;
use super::shape::{DisRange, RayIntersection, Shape};

#[derive(Debug)]
pub struct Scene {
    entities: Vec<Entity>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn add<S: Shape>(&mut self, shape: S) -> Id {
        let id = Id::new(self.entities.len());
        self.entities.push(Entity::new(id, shape));
        id
    }

    pub fn find_intersection(
        &self,
        ray_trace: &RayTrace,
        mut range: DisRange,
    ) -> Option<RayIntersection> {
        let mut closet: Option<RayIntersection> = None;
        for entity in &self.entities {
            if let Some(closet) = &closet {
                range = range.shrink_end(closet.distance());
            }
            let Some(intersection) = entity.hit(ray_trace, range) else {
                continue;
            };
            if let Some(closet) = &mut closet {
                if intersection.distance() < closet.distance() {
                    *closet = intersection
                }
            } else {
                closet = Some(intersection)
            }
        }
        closet
    }
}
