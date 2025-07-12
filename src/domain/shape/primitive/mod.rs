mod plane;
mod polygon;
mod sphere;
mod triangle;

pub use plane::Plane;
pub use polygon::{Polygon, TryNewPolygonError};
pub use sphere::{Sphere, TryNewSphereError};
pub use triangle::{Triangle, TryNewTriangleError};
