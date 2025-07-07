use smallvec::SmallVec;

use crate::domain::geometry::Point;
use crate::domain::ray::Ray;

use super::material::Material;
use super::shape::{CreateMeshShapeError, DisRange, Mesh, RayIntersection, Shape};
use super::{EntityId, EntityPool};

#[derive(Debug)]
pub struct Scene {
    entities: EntityPool,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            entities: EntityPool::new(),
        }
    }

    pub fn add<S: Shape, M: Material>(&mut self, shape: S, material: M) -> EntityId {
        self.entities.add_composition(shape, material)
    }

    pub fn add_mesh<M: Material>(
        &mut self,
        vertices: SmallVec<[Point; 8]>,
        vertex_indices: Vec<SmallVec<[usize; 3]>>,
        material: M,
    ) -> Result<(), CreateMeshShapeError> {
        let (triangles, polygons) = Mesh::shapes(vertices, vertex_indices, material)?;
        for triangle in triangles {
            self.entities.add_mesh_triangle(triangle);
        }
        for polygon in polygons {
            self.entities.add_mesh_polygon(polygon);
        }
        Ok(())
    }

    pub fn find_intersection(
        &self,
        ray: &Ray,
        mut range: DisRange,
    ) -> Option<(RayIntersection, &dyn Material)> {
        let mut closet: Option<RayIntersection> = None;
        let mut hit: Option<EntityId> = None;

        for id in self.entities.get_ids() {
            let shape = self.entities.get_shape(*id).unwrap();
            if let Some(closet) = &closet {
                range = range.shrink_end(closet.distance());
            }
            let Some(intersection) = shape.hit(ray, range) else {
                continue;
            };
            if let Some(closet) = &mut closet {
                if intersection.distance() < closet.distance() {
                    *closet = intersection;
                    hit = Some(*id);
                }
            } else {
                closet = Some(intersection);
                hit = Some(*id);
            }
        }

        closet
            .zip(hit)
            .map(|(closet, id)| (closet, self.entities.get_material(id).unwrap()))
    }
}
