use std::sync::Arc;

use smallvec::SmallVec;
use snafu::prelude::*;

use crate::domain::math::geometry::{AllTransformation, Point};
use crate::domain::shape::primitive::{MeshPolygon, MeshTriangle, Polygon, Triangle};
use crate::domain::shape::util::{ShapeConstructor, ShapeContainer, ShapeId};

use super::data::{OutOfBoundSnafu, PolygonSnafu, TriangleSnafu};
use super::{MeshData, TryNewMeshError};

type TriangleIndices = (u32, u32, u32);
type PolygonIndices = SmallVec<[u32; 5]>;

#[derive(Debug, Clone)]
pub struct MeshConstructor {
    vertices: Arc<[Point]>,
    triangles: Arc<[TriangleIndices]>,
    polygons: Arc<[PolygonIndices]>,
}

impl MeshConstructor {
    pub fn new(
        vertices: Vec<Point>,
        vertex_indices: Vec<Vec<usize>>,
    ) -> Result<Self, TryNewMeshError> {
        Self::new_shared(vertices.into(), vertex_indices)
    }

    pub fn new_shared(
        vertices: Arc<[Point]>,
        vertex_indices: Vec<Vec<usize>>,
    ) -> Result<Self, TryNewMeshError> {
        let (triangles, polygons) = Self::validate_and_create_shapes(&vertices, vertex_indices)?;
        Ok(Self {
            vertices,
            triangles: triangles.into(),
            polygons: polygons.into(),
        })
    }

    fn validate_and_create_shapes(
        vertices: &[Point],
        mut vertex_indices: Vec<Vec<usize>>,
    ) -> Result<(Vec<TriangleIndices>, Vec<PolygonIndices>), TryNewMeshError> {
        let triangle_indices = vertex_indices
            .extract_if(.., |s| s.len() == 3)
            .collect::<Vec<_>>();
        let triangles = Self::validate_and_create_triangles(vertices, &triangle_indices)?;

        let polygon_indices = vertex_indices;
        let polygons = Self::validate_and_create_polygons(vertices, &polygon_indices)?;

        Ok((triangles, polygons))
    }

    fn validate_and_create_triangles(
        vertices: &[Point],
        triangles: &[Vec<usize>],
    ) -> Result<Vec<TriangleIndices>, TryNewMeshError> {
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
    ) -> Result<Vec<PolygonIndices>, TryNewMeshError> {
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

    pub fn construct_impl(
        self,
        transformation: Option<AllTransformation>,
        inv_transformation: Option<AllTransformation>,
    ) -> (Vec<MeshTriangle>, Vec<MeshPolygon>) {
        let data = Arc::new(MeshData {
            vertices: Arc::clone(&self.vertices),
            triangles: Arc::clone(&self.triangles),
            polygons: Arc::clone(&self.polygons),
            transformation,
            inv_transformation,
        });

        let mesh_triangles = (0..data.triangles.len())
            .map(|index| MeshTriangle::new(data.clone(), index))
            .collect();

        let mesh_polygons = (0..data.polygons.len())
            .map(|index| MeshPolygon::new(data.clone(), index))
            .collect();

        (mesh_triangles, mesh_polygons)
    }
}

impl ShapeConstructor for MeshConstructor {
    fn construct<C: ShapeContainer>(self, container: &mut C) -> Vec<ShapeId> {
        let (triangles, polygons) = self.construct_impl(None, None);

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

#[cfg(test)]
mod tests {
    use crate::domain::math::numeric::Val;

    use super::*;

    #[test]
    fn mesh_constructor_new_succeeds() {
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
        .construct_impl(None, None);

        assert_eq!(triangles.len(), 4);
        assert_eq!(polygons.len(), 1);
    }
}
