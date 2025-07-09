use std::collections::HashMap;
use std::sync::Arc;

use smallvec::SmallVec;
use snafu::prelude::*;

use crate::domain::geometry::{Point, Product};
use crate::domain::ray::Ray;

use super::{
    BoundingBox, DisRange, Polygon, RayIntersection, Shape, ShapeKind, Triangle,
    TryNewPolygonError, TryNewTriangleError,
};

#[derive(Debug)]
pub struct Mesh {
    vertices: SmallVec<[Point; 8]>,
}

impl Mesh {
    fn new(vertices: SmallVec<[Point; 8]>) -> Self {
        Self { vertices }
    }

    pub fn shapes(
        vertices: SmallVec<[Point; 8]>,
        mut vertex_indices: Vec<SmallVec<[usize; 3]>>,
    ) -> Result<(Vec<MeshTriangle>, Vec<MeshPolygon>), CreateMeshShapeError> {
        Self::validate_vertex_uniqueness(&vertices)?;
        let data = Arc::new(Mesh::new(vertices));

        let triangles = vertex_indices.extract_if(.., |s| s.len() == 3).collect();
        let polygons = vertex_indices;

        Ok((
            Self::create_triangles(&data, triangles)?,
            Self::create_polygon(&data, polygons)?,
        ))
    }

    fn validate_vertex_uniqueness(vertices: &[Point]) -> Result<(), CreateMeshShapeError> {
        let mut buc = HashMap::new();
        for (i, v) in vertices.iter().enumerate() {
            if let Some(former) = buc.insert(*v, i) {
                return Err(CreateMeshShapeError::DuplicatedVertices { former, latter: i });
            }
        }
        Ok(())
    }

    fn create_triangles(
        data: &Arc<Mesh>,
        triangles: Vec<SmallVec<[usize; 3]>>,
    ) -> Result<Vec<MeshTriangle>, CreateMeshShapeError> {
        let mut res = Vec::with_capacity(triangles.len());

        for (surface, triangle) in triangles.into_iter().enumerate() {
            let vertices = (triangle.iter())
                .map(|&index| (index, data.vertices.get(index)))
                .map(|(index, res)| res.context(OutOfBoundSnafu { surface, index }))
                .collect::<Result<SmallVec<[_; 3]>, _>>()?;

            assert!(vertices.len() == 3);
            Triangle::validate_vertices(vertices[0], vertices[1], vertices[2])
                .context(TriangleSnafu { surface })?;

            assert!(triangle.len() == 3);
            res.push(MeshTriangle {
                data: data.clone(),
                vertex0: triangle[0],
                vertex1: triangle[1],
                vertex2: triangle[2],
            });
        }

        Ok(res)
    }

    fn create_polygon(
        data: &Arc<Mesh>,
        polygons: Vec<SmallVec<[usize; 3]>>,
    ) -> Result<Vec<MeshPolygon>, CreateMeshShapeError> {
        let mut res = Vec::with_capacity(polygons.len());

        for (surface, polygon) in polygons.into_iter().enumerate() {
            let vertices = (polygon.iter())
                .map(|&index| (index, data.vertices.get(index).cloned()))
                .map(|(index, res)| res.context(OutOfBoundSnafu { surface, index }))
                .collect::<Result<Vec<_>, _>>()?;

            let _ = Polygon::new(vertices).context(PolygonSnafu { surface })?;

            res.push(MeshPolygon {
                data: data.clone(),
                vertex_indices: polygon.into_iter().collect(),
            });
        }

        Ok(res)
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CreateMeshShapeError {
    #[snafu(display("mesh has duplicated vertices {former} and {latter}"))]
    DuplicatedVertices { former: usize, latter: usize },
    #[snafu(display("index {index} for vertex in surface {surface} is out of bound"))]
    OutOfBound { surface: usize, index: usize },
    #[snafu(display("could not create mesh surface {surface} as triangle"))]
    Triangle {
        surface: usize,
        source: TryNewTriangleError,
    },
    #[snafu(display("could not create mesh surface {surface} as polygon"))]
    Polygon {
        surface: usize,
        source: TryNewPolygonError,
    },
}

#[derive(Debug, Clone)]
pub struct MeshTriangle {
    data: Arc<Mesh>,
    vertex0: usize,
    vertex1: usize,
    vertex2: usize,
}

impl Shape for MeshTriangle {
    fn shape_kind(&self) -> ShapeKind {
        ShapeKind::Triangle
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        let v0 = (self.data.vertices.get(self.vertex0))
            .expect("vertex0 index has been checked during mesh construction");
        let v1 = (self.data.vertices.get(self.vertex1))
            .expect("vertex1 index has been checked during mesh construction");
        let v2 = (self.data.vertices.get(self.vertex2))
            .expect("vertex2 index has been checked during mesh construction");
        Triangle::calc_ray_intersection(ray, range, v0, v1, v2)
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let v0 = (self.data.vertices.get(self.vertex0))
            .expect("vertex0 index has been checked during mesh construction");
        let v1 = (self.data.vertices.get(self.vertex1))
            .expect("vertex1 index has been checked during mesh construction");
        let v2 = (self.data.vertices.get(self.vertex2))
            .expect("vertex2 index has been checked during mesh construction");
        let min = v0.component_min(v1).component_min(v2);
        let max = v0.component_max(v1).component_max(v2);
        Some(BoundingBox::new(min, max))
    }
}

#[derive(Debug, Clone)]
pub struct MeshPolygon {
    data: Arc<Mesh>,
    vertex_indices: SmallVec<[usize; 6]>,
}

impl Shape for MeshPolygon {
    fn shape_kind(&self) -> ShapeKind {
        ShapeKind::Polygon
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        let vertices = (self.vertex_indices.iter())
            .map(|index| {
                (self.data.vertices.get(*index))
                    .expect("index has been checked during mesh construction")
            })
            .collect::<SmallVec<[_; 6]>>();

        assert!(vertices.len() > 3);
        let normal = (*vertices[1] - *vertices[0])
            .cross(*vertices[2] - *vertices[1])
            .normalize()
            .expect("normal existence has been checked during mesh construction");

        Polygon::calc_ray_intersection(ray, range, &vertices, &normal)
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let mut vertices = (self.vertex_indices.iter()).map(|index| {
            (self.data.vertices.get(*index))
                .expect("index has been checked during mesh construction")
        });
        let init = *vertices.next().expect("init should exist");
        let (min, max) = vertices.fold((init, init), |(min, max), vertex| {
            (min.component_min(vertex), max.component_max(vertex))
        });
        Some(BoundingBox::new(min, max))
    }
}

#[cfg(test)]
mod tests {
    use smallvec::smallvec;

    use crate::domain::geometry::Val;

    use super::*;

    #[test]
    fn mesh_shapes_succeeds() {
        let (triangles, polygons) = Mesh::shapes(
            smallvec![
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(2.0)),
            ],
            vec![
                smallvec![0, 1, 2, 3],
                smallvec![0, 1, 4],
                smallvec![1, 2, 4],
                smallvec![2, 3, 4],
                smallvec![3, 1, 4],
            ],
        )
        .unwrap();

        assert_eq!(triangles.len(), 4);
        assert_eq!(polygons.len(), 1);
    }

    #[test]
    fn mesh_bounding_box_succeeds() {
        let (triangles, polygons) = Mesh::shapes(
            smallvec![
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(2.0)),
            ],
            vec![smallvec![0, 1, 2, 3], smallvec![0, 1, 4]],
        )
        .unwrap();

        assert_eq!(
            triangles[0].bounding_box(),
            Some(BoundingBox::new(
                Point::new(Val(-1.0), Val(0.0), Val(0.0)),
                Point::new(Val(1.0), Val(1.0), Val(2.0)),
            )),
        );

        assert_eq!(
            polygons[0].bounding_box(),
            Some(BoundingBox::new(
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
            )),
        );
    }
}
