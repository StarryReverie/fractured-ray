use std::sync::Arc;

use smallvec::SmallVec;
use snafu::prelude::*;

use crate::domain::math::geometry::Point;
use crate::domain::math::transformation::Sequential;
use crate::domain::shape::primitive::{TryNewPolygonError, TryNewTriangleError};

#[derive(Debug, Clone)]
pub struct MeshData {
    vertices: Arc<[Point]>,
    triangles: Arc<[(u32, u32, u32)]>,
    polygons: Arc<[SmallVec<[u32; 5]>]>,
    transformation: Option<Sequential>,
}

impl MeshData {
    pub fn vertices(&self) -> &[Point] {
        &self.vertices
    }

    pub fn triangles(&self) -> &[(u32, u32, u32)] {
        &self.triangles
    }

    pub fn polygons(&self) -> &[SmallVec<[u32; 5]>] {
        &self.polygons
    }

    pub fn transformation(&self) -> Option<&Sequential> {
        self.transformation.as_ref()
    }
}

impl MeshData {
    pub fn new(
        vertices: Arc<[Point]>,
        triangles: Arc<[(u32, u32, u32)]>,
        polygons: Arc<[SmallVec<[u32; 5]>]>,
        transformation: Option<Sequential>,
    ) -> Self {
        Self {
            vertices,
            triangles,
            polygons,
            transformation,
        }
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[snafu(visibility(pub(super)))]
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
