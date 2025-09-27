use std::error::Error;
use std::io::Error as IoError;
use std::path::PathBuf;

use snafu::prelude::*;

use crate::domain::image::core::Image;

pub trait ImageResource: Send + Sync {
    fn load(&self) -> Result<Image, LoadImageError>;

    fn save(&self, image: &Image) -> Result<(), SaveImageError>;
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)), context(suffix(LoadSnafu)))]
#[non_exhaustive]
pub enum LoadImageError {
    #[snafu(display("could not find image `{}`", path.display()))]
    NotFound { path: PathBuf },
    #[snafu(display("IO operation failed when load image `{}`", path.display()))]
    Io { path: PathBuf, source: IoError },
    #[snafu(whatever, display("could not load image: {}", message))]
    Unknown {
        message: String,
        #[snafu(source(from(Box<dyn Error + Send + Sync>, Some)))]
        source: Option<Box<dyn Error + Send + Sync>>,
    },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)), context(suffix(SaveSnafu)))]
#[non_exhaustive]
pub enum SaveImageError {
    #[snafu(display("IO operation failed when load image `{}`", path.display()))]
    Io { path: PathBuf, source: IoError },
    #[snafu(whatever, display("could not load image: {}", message))]
    Unknown {
        message: String,
        #[snafu(source(from(Box<dyn Error + Send + Sync>, Some)))]
        source: Option<Box<dyn Error + Send + Sync>>,
    },
}
