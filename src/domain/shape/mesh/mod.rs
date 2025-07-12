mod constructor;
mod data;
mod primitive;

pub use constructor::MeshConstructor;
pub use data::{MeshData, TryNewMeshError};
pub use primitive::{MeshPolygon, MeshTriangle};
