pub mod material;
pub mod shape;

mod entity;
mod scene;

pub use entity::{EntityId, EntityPool, MaterialContainer, MaterialId, ShapeContainer, ShapeId};
pub use scene::{Scene, SceneBuilder};
