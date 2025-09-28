use std::fmt::Debug;
use std::sync::Arc;

use crate::domain::image::core::Image;

use super::LoadImageError;

pub trait ImageRegistry: Debug + Send + Sync {
    fn get(&self, name: &str) -> Result<Arc<Image>, LoadImageError>;
}
