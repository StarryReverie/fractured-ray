use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use getset::{Getters, WithSetters};
use snafu::prelude::*;

use crate::domain::material::def::{DynMaterial, MaterialKind};
use crate::domain::math::transformation::Sequential;
use crate::domain::scene::entity::EntitySceneBuilder;
use crate::domain::shape::mesh::TryNewMeshError;

pub trait EntityModelLoader: Send + Sync {
    fn load(
        &self,
        builder: &mut dyn EntitySceneBuilder,
        config: EntityModelLoaderConfiguration,
    ) -> Result<(), LoadEntityModelError>;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Getters, WithSetters)]
pub struct EntityModelLoaderConfiguration {
    #[getset(get = "pub", set_with = "pub")]
    transformation: Sequential,
    #[getset(get = "pub", set_with = "pub")]
    materials: HashMap<String, DynMaterial>,
}

impl EntityModelLoaderConfiguration {
    pub fn add_material<S, M>(mut self, name: S, material: M) -> Self
    where
        S: Into<String>,
        M: Into<DynMaterial>,
    {
        self.materials.insert(name.into(), material.into());
        self
    }
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
#[snafu(visibility(pub))]
pub enum LoadEntityModelError {
    #[snafu(display(
        "encountered invalid mesh `{mesh_name}` from `{}`",
        display_optional_path(path)
    ))]
    InvalidMesh {
        path: Option<PathBuf>,
        mesh_name: String,
        source: TryNewMeshError,
    },
    #[snafu(display(
        "parameters of material {material_name} ({material_kind:?}) are incorrectly configured"
    ))]
    InvalidMaterial {
        material_kind: MaterialKind,
        material_name: String,
        source: Box<dyn Error + Send + Sync>,
    },
    #[snafu(display(
        "no material is specified for mesh `{mesh_name}` from `{}`",
        display_optional_path(path)
    ))]
    UnspecifiedMaterial {
        path: Option<PathBuf>,
        mesh_name: String,
    },
    #[snafu(display("material `{material_name}` is missing"))]
    MissingMaterial { material_name: String },
    #[snafu(whatever, display("could not load models: {message}"))]
    Unknown {
        message: String,
        source: Option<Box<dyn Error + Send + Sync>>,
    },
}

fn display_optional_path(path: &Option<PathBuf>) -> String {
    path.as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "<in-memory>".into())
}
