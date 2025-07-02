use crate::domain::ray::RayTrace;

use super::Entity;
use super::entity::Id;
use super::material::Material;
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

    pub fn add<S: Shape, M: Material>(&mut self, shape: S, material: M) -> Id {
        let id = Id::new(self.entities.len());
        self.entities.push(Entity::new(id, shape, material));
        id
    }

    pub fn find_intersection(
        &self,
        ray_trace: &RayTrace,
        mut range: DisRange,
    ) -> Option<(RayIntersection, &Entity)> {
        let mut closet: Option<RayIntersection> = None;
        let mut hit: Option<&Entity> = None;

        for entity in &self.entities {
            if let Some(closet) = &closet {
                range = range.shrink_end(closet.distance());
            }
            let Some(intersection) = entity.hit(ray_trace, range) else {
                continue;
            };
            if let Some(closet) = &mut closet {
                if intersection.distance() < closet.distance() {
                    *closet = intersection;
                    hit = Some(entity);
                }
            } else {
                closet = Some(intersection);
                hit = Some(entity);
            }
        }

        closet.zip(hit)
    }
}
