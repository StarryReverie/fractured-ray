use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use snafu::prelude::*;

use crate::domain::image::core::Image;
use crate::domain::image::external::{ImageRegistry, ImageResource, LoadImageError};

use super::{PngImageResource, PpmImageResource};

#[derive(Debug)]
pub struct FileSystemImageRegistry {
    images: RwLock<HashMap<String, Arc<Image>>>,
}

impl FileSystemImageRegistry {
    pub fn new() -> Self {
        Self {
            images: RwLock::new(HashMap::new()),
        }
    }
}

impl ImageRegistry for FileSystemImageRegistry {
    fn get(&self, name: &str) -> Result<Arc<Image>, LoadImageError> {
        if let Some(image) = self.images.read().unwrap().get(name) {
            return Ok(Arc::clone(image));
        }

        let name_lowercase = name.to_lowercase();
        let image = if name_lowercase.ends_with(".png") {
            Arc::new(PngImageResource::new(name).load()?)
        } else if name_lowercase.ends_with(".ppm") {
            Arc::new(PpmImageResource::new(name).load()?)
        } else {
            whatever!("the type of image `{}` is unsupported", name);
        };

        let mut images = self.images.write().unwrap();
        images.insert(name.into(), Arc::clone(&image));
        Ok(image)
    }
}

#[derive(Debug)]
pub struct DirectoryImageRegistryProxy {
    inner: Arc<dyn ImageRegistry>,
    directory: PathBuf,
}

impl DirectoryImageRegistryProxy {
    pub fn new<P>(
        inner: Arc<dyn ImageRegistry>,
        directory: P,
    ) -> Result<Self, TryNewDirectoryImageRegistryProxyError>
    where
        P: Into<PathBuf>,
    {
        let directory = directory.into();
        ensure!(
            directory.is_dir(),
            InvalidDirectorySnafu { path: directory }
        );
        Ok(Self { inner, directory })
    }

    fn resolve_path(&self, name: &str) -> PathBuf {
        let path: &Path = name.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.directory.join(path)
        }
    }
}

impl ImageRegistry for DirectoryImageRegistryProxy {
    fn get(&self, name: &str) -> Result<Arc<Image>, LoadImageError> {
        let abs_path = self.resolve_path(name);
        self.inner.get(&abs_path.display().to_string())
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
pub enum TryNewDirectoryImageRegistryProxyError {
    #[snafu(display("{} is not a valid directory", path.display()))]
    InvalidDirectory { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyRegistry;

    impl ImageRegistry for DummyRegistry {
        fn get(&self, _: &str) -> Result<Arc<Image>, LoadImageError> {
            unreachable!()
        }
    }

    #[test]
    fn test_resolve_path_relative_and_absolute() {
        let dir = PathBuf::from("./");
        let registry =
            DirectoryImageRegistryProxy::new(Arc::new(DummyRegistry), dir.clone()).unwrap();

        let absolute = registry.resolve_path("/foo/bar.png");
        assert_eq!(absolute, PathBuf::from("/foo/bar.png"));

        let relative = registry.resolve_path("baz.png");
        assert_eq!(relative, dir.join("baz.png"));
    }
}
