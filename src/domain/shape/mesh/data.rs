use std::sync::Arc;

use smallvec::SmallVec;
use snafu::prelude::*;

use crate::domain::math::geometry::Point;
use crate::domain::math::transformation::Sequential;
use crate::domain::shape::primitive::{Polygon, Triangle, TryNewPolygonError, TryNewTriangleError};
use crate::domain::texture::def::UvCoordinate;

pub type TriangleIndices = (u32, u32, u32);
pub type PolygonIndices = SmallVec<[u32; 5]>;

#[derive(Debug, Clone)]
pub struct MeshData {
    vertices: MeshDataComponent<Point>,
    textures: Option<MeshDataComponent<UvCoordinate>>,
    transformation: Option<Sequential>,
}

impl MeshData {
    pub fn new(
        vertices: MeshDataComponent<Point>,
        textures: Option<MeshDataComponent<UvCoordinate>>,
        transformation: Option<Sequential>,
    ) -> Self {
        Self {
            vertices,
            textures,
            transformation,
        }
    }

    #[inline]
    pub fn vertices(&self) -> &MeshDataComponent<Point> {
        &self.vertices
    }

    #[inline]
    pub fn textures(&self) -> Option<&MeshDataComponent<UvCoordinate>> {
        self.textures.as_ref()
    }

    #[inline]
    pub fn transformation(&self) -> Option<&Sequential> {
        self.transformation.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct MeshDataComponent<T>
where
    T: Send + Sync,
{
    data: Arc<[T]>,
    triangles: Arc<[TriangleIndices]>,
    polygons: Arc<[PolygonIndices]>,
}

impl<T> MeshDataComponent<T>
where
    T: Send + Sync,
{
    fn new_impl(
        data: Arc<[T]>,
        triangles: Arc<[TriangleIndices]>,
        polygons: Arc<[PolygonIndices]>,
    ) -> Self {
        Self {
            data,
            triangles,
            polygons,
        }
    }

    #[inline]
    pub fn data(&self) -> &[T] {
        &self.data
    }

    #[inline]
    pub fn triangles(&self) -> &[TriangleIndices] {
        &self.triangles
    }

    #[inline]
    pub fn polygons(&self) -> &[PolygonIndices] {
        &self.polygons
    }
}

impl MeshDataComponent<Point> {
    pub fn new<V>(vertices: V, indices: Vec<Vec<usize>>) -> Result<Self, TryNewMeshError>
    where
        V: Into<Arc<[Point]>>,
    {
        let vertices = vertices.into();
        let triangles = Self::create_triangles(&vertices, &indices)?.into();
        let polygons = Self::create_polygons(&vertices, &indices)?.into();
        Ok(Self::new_impl(vertices, triangles, polygons))
    }

    fn create_triangles(
        vertices: &[Point],
        indices: &[Vec<usize>],
    ) -> Result<Vec<TriangleIndices>, TryNewMeshError> {
        let mut res = Vec::with_capacity(indices.len());
        for (face, triangle) in indices.iter().enumerate().filter(|(_, s)| s.len() == 3) {
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
        res.shrink_to_fit();
        Ok(res)
    }

    fn create_polygons(
        vertices: &[Point],
        polygons: &[Vec<usize>],
    ) -> Result<Vec<PolygonIndices>, TryNewMeshError> {
        let mut res = Vec::with_capacity(polygons.len());
        for (face, polygon) in polygons.iter().enumerate().filter(|(_, s)| s.len() != 3) {
            let vertices = (polygon.iter())
                .map(|&index| (index, vertices.get(index).cloned()))
                .map(|(index, res)| res.context(OutOfBoundSnafu { face, index }))
                .collect::<Result<Vec<_>, _>>()?;

            let _ = Polygon::new(vertices).context(PolygonSnafu { face })?;

            res.push(polygon.iter().map(|&i| i as u32).collect());
        }
        res.shrink_to_fit();
        Ok(res)
    }
}

impl MeshDataComponent<UvCoordinate> {
    pub fn new<U>(
        uvs: U,
        indices: Vec<Vec<usize>>,
        num_triangles: usize,
        num_polygons: usize,
    ) -> Result<Self, TryAddMeshUvCoordinateError>
    where
        U: Into<Arc<[UvCoordinate]>>,
    {
        let uvs = uvs.into();
        let triangles = Self::create_triangles_uv(&uvs, &indices, num_triangles)?.into();
        let polygons = Self::create_polygons_uv(&uvs, &indices, num_polygons)?.into();
        Ok(Self::new_impl(uvs, triangles, polygons))
    }

    fn create_triangles_uv(
        uvs: &[UvCoordinate],
        indices: &[Vec<usize>],
        expected_num: usize,
    ) -> Result<Vec<TriangleIndices>, TryAddMeshUvCoordinateError> {
        let mut res = Vec::with_capacity(indices.len());
        for (face, triangle) in indices.iter().enumerate().filter(|(_, s)| s.len() == 3) {
            if let Some(index) = triangle.iter().cloned().find(|&index| index >= uvs.len()) {
                return OutOfBoundUvSnafu { face, index }.fail();
            }

            assert!(triangle.len() == 3);
            res.push((triangle[0] as u32, triangle[1] as u32, triangle[2] as u32));
        }
        ensure!(res.len() == expected_num, MismatchedNumberUvSnafu);
        res.shrink_to_fit();
        Ok(res)
    }

    fn create_polygons_uv(
        uvs: &[UvCoordinate],
        indices: &[Vec<usize>],
        expected_num: usize,
    ) -> Result<Vec<PolygonIndices>, TryAddMeshUvCoordinateError> {
        let mut res = Vec::with_capacity(indices.len());
        for (face, polygon) in indices.iter().enumerate().filter(|(_, s)| s.len() != 3) {
            if let Some(index) = polygon.iter().cloned().find(|&index| index >= uvs.len()) {
                return OutOfBoundUvSnafu { face, index }.fail();
            }

            res.push(polygon.iter().map(|&i| i as u32).collect());
        }
        ensure!(res.len() == expected_num, MismatchedNumberUvSnafu);
        res.shrink_to_fit();
        Ok(res)
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewMeshError {
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

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[snafu(context(suffix(UvSnafu)))]
#[non_exhaustive]
pub enum TryAddMeshUvCoordinateError {
    #[snafu(display("the number of UV coordinates is not same as vertices"))]
    MismatchedNumber,
    #[snafu(display("index {index} for UV coordinate in face {face} is out of bound"))]
    OutOfBound { face: usize, index: usize },
}
