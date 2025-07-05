mod plane;
mod polygon;
mod shape;
mod sphere;
mod triangle;

pub use plane::Plane;
pub use polygon::{Polygon, TryNewPolygonError};
pub use shape::{DisRange, RayIntersection, Shape, SurfaceSide};
pub use sphere::{Sphere, TryNewSphereError};
pub use triangle::{Triangle, TryNewTriangleError};
