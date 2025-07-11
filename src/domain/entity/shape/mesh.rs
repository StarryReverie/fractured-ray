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
pub struct MeshConstructor {
    vertices: Arc<[Point]>,
    triangles: Arc<[(u32, u32, u32)]>,
    polygons: Arc<[SmallVec<[u32; 5]>]>,
}

impl MeshConstructor {
    pub fn new(
        vertices: Vec<Point>,
        mut vertex_indices: Vec<Vec<usize>>,
    ) -> Result<Self, TryNewMeshError> {
        Self::validate_vertex_uniqueness(&vertices)?;

        let triangle_indices = vertex_indices
            .extract_if(.., |s| s.len() == 3)
            .collect::<Vec<_>>();
        let triangles = Self::validate_and_create_triangles(&vertices, &triangle_indices)?;

        let polygon_indices = vertex_indices;
        let polygons = Self::validate_and_create_polygons(&vertices, &polygon_indices)?;

        Ok(Self {
            vertices: vertices.into(),
            triangles: triangles.into(),
            polygons: polygons.into(),
        })
    }

    fn validate_vertex_uniqueness(vertices: &[Point]) -> Result<(), TryNewMeshError> {
        let mut buc = HashMap::new();
        for (i, v) in vertices.iter().enumerate() {
            if let Some(former) = buc.insert(*v, i) {
                return Err(TryNewMeshError::DuplicatedVertices { former, latter: i });
            }
        }
        Ok(())
    }

    fn validate_and_create_triangles(
        vertices: &[Point],
        triangles: &[Vec<usize>],
    ) -> Result<Vec<(u32, u32, u32)>, TryNewMeshError> {
        let mut res = Vec::with_capacity(triangles.len());

        for (face, triangle) in triangles.iter().enumerate() {
            let vertices = (triangle.iter())
                .map(|&index| (index, vertices.get(index)))
                .map(|(index, res)| res.context(OutOfBoundSnafu { face, index }))
                .collect::<Result<SmallVec<[_; 3]>, _>>()?;

            assert!(vertices.len() == 3);
            Triangle::validate_vertices(vertices[0], vertices[1], vertices[2])
                .context(TriangleSnafu { face })?;

            assert!(triangle.len() == 3);
            res.push((triangle[0] as u32, triangle[1] as u32, triangle[2] as u32));
        }

        Ok(res)
    }

    fn validate_and_create_polygons(
        vertices: &[Point],
        polygons: &[Vec<usize>],
    ) -> Result<Vec<SmallVec<[u32; 5]>>, TryNewMeshError> {
        let mut res = Vec::with_capacity(polygons.len());

        for (face, polygon) in polygons.iter().enumerate() {
            let vertices = (polygon.iter())
                .map(|&index| (index, vertices.get(index).cloned()))
                .map(|(index, res)| res.context(OutOfBoundSnafu { face, index }))
                .collect::<Result<Vec<_>, _>>()?;

            let _ = Polygon::new(vertices).context(PolygonSnafu { face })?;

            res.push(polygon.iter().map(|&i| i as u32).collect());
        }

        Ok(res)
    }

    pub fn construct(self) -> (Vec<MeshTriangle>, Vec<MeshPolygon>) {
        let data = Arc::new(Mesh {
            vertices: self.vertices,
            triangles: self.triangles,
            polygons: self.polygons,
        });

        let mesh_triangles = (0..data.triangles.len())
            .map(|index| MeshTriangle {
                data: data.clone(),
                index,
            })
            .collect();

        let mesh_polygons = (0..data.polygons.len())
            .map(|index| MeshPolygon {
                data: data.clone(),
                index,
            })
            .collect();

        (mesh_triangles, mesh_polygons)
    }
}

#[derive(Debug)]
pub struct Mesh {
    vertices: Arc<[Point]>,
    triangles: Arc<[(u32, u32, u32)]>,
    polygons: Arc<[SmallVec<[u32; 5]>]>,
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewMeshError {
    #[snafu(display("mesh has duplicated vertices {former} and {latter}"))]
    DuplicatedVertices { former: usize, latter: usize },
    #[snafu(display("index {index} for vertex in face {face} is out of bound"))]
    OutOfBound { face: usize, index: usize },
    #[snafu(display("could not create mesh face {face} as triangle"))]
    Triangle {
        face: usize,
        source: TryNewTriangleError,
    },
    #[snafu(display("could not create mesh face {face} as polygon"))]
    Polygon {
        face: usize,
        source: TryNewPolygonError,
    },
}

#[derive(Debug, Clone)]
pub struct MeshTriangle {
    data: Arc<Mesh>,
    index: usize,
}

impl MeshTriangle {
    fn get_vertices(&self) -> (&Point, &Point, &Point) {
        let vertices = &self.data.vertices;
        let triangles = &self.data.triangles;
        let v0 = &vertices[triangles[self.index].0 as usize];
        let v1 = &vertices[triangles[self.index].1 as usize];
        let v2 = &vertices[triangles[self.index].2 as usize];
        (v0, v1, v2)
    }
}

impl Shape for MeshTriangle {
    fn shape_kind(&self) -> ShapeKind {
        ShapeKind::MeshTriangle
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        let (v0, v1, v2) = self.get_vertices();
        Triangle::calc_ray_intersection(ray, range, v0, v1, v2)
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let (v0, v1, v2) = self.get_vertices();
        let min = v0.component_min(v1).component_min(v2);
        let max = v0.component_max(v1).component_max(v2);
        Some(BoundingBox::new(min, max))
    }
}

#[derive(Debug, Clone)]
pub struct MeshPolygon {
    data: Arc<Mesh>,
    index: usize,
}

impl MeshPolygon {
    fn get_vertices(&self) -> SmallVec<[&Point; 5]> {
        let vertices = &self.data.vertices;
        let polygons = &self.data.polygons;
        polygons[self.index]
            .iter()
            .map(|index| &vertices[*index as usize])
            .collect::<SmallVec<[_; 5]>>()
    }
}

impl Shape for MeshPolygon {
    fn shape_kind(&self) -> ShapeKind {
        ShapeKind::MeshPolygon
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        let vertices = self.get_vertices();

        assert!(vertices.len() > 3);
        let normal = (*vertices[1] - *vertices[0])
            .cross(*vertices[2] - *vertices[1])
            .normalize()
            .expect("normal existence has been checked during mesh construction");

        Polygon::calc_ray_intersection(ray, range, &vertices, &normal)
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let mut vertices = self.get_vertices().into_iter();
        let init = *vertices.next().expect("init should exist");
        let (min, max) = vertices.fold((init, init), |(min, max), vertex| {
            (min.component_min(vertex), max.component_max(vertex))
        });
        Some(BoundingBox::new(min, max))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::geometry::Val;

    use super::*;

    #[test]
    fn mesh_shapes_succeeds() {
        let (triangles, polygons) = MeshConstructor::new(
            vec![
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(2.0)),
            ],
            vec![
                vec![0, 1, 2, 3],
                vec![0, 1, 4],
                vec![1, 2, 4],
                vec![2, 3, 4],
                vec![3, 1, 4],
            ],
        )
        .unwrap()
        .construct();

        assert_eq!(triangles.len(), 4);
        assert_eq!(polygons.len(), 1);
    }

    #[test]
    fn mesh_bounding_box_succeeds() {
        let (triangles, polygons) = MeshConstructor::new(
            vec![
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(2.0)),
            ],
            vec![vec![0, 1, 2, 3], vec![0, 1, 4]],
        )
        .unwrap()
        .construct();

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
