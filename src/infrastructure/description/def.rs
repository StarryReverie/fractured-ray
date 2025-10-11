use crate::domain::camera::Camera;
use crate::domain::renderer::CoreRendererConfiguration;
use crate::domain::scene::entity::EntityScene;
use crate::domain::scene::volume::VolumeScene;

use super::error::LoadDescriptionError;

/// Trait for loading scene descriptions from configuration files
pub trait DescriptionLoader {
    /// Load a scene description
    fn load(&self) -> Result<Description, LoadDescriptionError>;
}

/// Complete scene description including renderer config, camera, and scenes
pub struct Description {
    pub renderer_config: CoreRendererConfiguration,
    pub camera: Camera,
    pub entity_scene: Box<dyn EntityScene>,
    pub volume_scene: Box<dyn VolumeScene>,
}

impl Description {
    pub fn new(
        renderer_config: CoreRendererConfiguration,
        camera: Camera,
        entity_scene: Box<dyn EntityScene>,
        volume_scene: Box<dyn VolumeScene>,
    ) -> Self {
        Self {
            renderer_config,
            camera,
            entity_scene,
            volume_scene,
        }
    }
}
