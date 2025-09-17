use std::sync::Arc;

use smallvec::SmallVec;
use snafu::prelude::*;

use crate::domain::math::geometry::{Sequential, Point};
use crate::domain::shape::primitive::{TryNewPolygonError, TryNewTriangleError};

#[derive(Debug)]
pub struct MeshData {
    pub(in crate::domain::shape) vertices: Arc<[Point]>,
    pub(in crate::domain::shape) triangles: Arc<[(u32, u32, u32)]>,
    pub(in crate::domain::shape) polygons: Arc<[SmallVec<[u32; 5]>]>,
    pub(in crate::domain::shape) transformation: Option<Sequential>,
    pub(in crate::domain::shape) inv_transformation: Option<Sequential>,
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
