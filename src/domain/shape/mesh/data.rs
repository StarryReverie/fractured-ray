use std::sync::Arc;

use smallvec::SmallVec;
use snafu::prelude::*;

use crate::domain::math::geometry::{AllTransformation, Point};
use crate::domain::shape::primitive::{TryNewPolygonError, TryNewTriangleError};

#[derive(Debug)]
pub struct MeshData {
    pub(super) vertices: Arc<[Point]>,
    pub(super) triangles: Arc<[(u32, u32, u32)]>,
    pub(super) polygons: Arc<[SmallVec<[u32; 5]>]>,
    pub(super) transformation: Option<AllTransformation>,
    pub(super) inv_transformation: Option<AllTransformation>,
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
