pub mod photon;
pub mod sampling;

mod intersection;
mod ray;

pub use intersection::{RayIntersection, SurfaceSide};
pub use ray::Ray;
