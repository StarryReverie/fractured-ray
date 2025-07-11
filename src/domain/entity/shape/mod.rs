mod bbox;
mod instance;
mod mesh;
mod plane;
mod polygon;
mod shape;
mod sphere;
mod triangle;

pub use bbox::BoundingBox;
pub use instance::Instance;
pub use mesh::{TryNewMeshError, MeshConstructor, MeshPolygon, MeshTriangle};
pub use plane::Plane;
pub use polygon::{Polygon, TryNewPolygonError};
pub use shape::{DisRange, RayIntersection, Shape, ShapeKind, SurfaceSide};
pub use sphere::{Sphere, TryNewSphereError};
pub use triangle::{Triangle, TryNewTriangleError};
