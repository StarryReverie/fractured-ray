use std::path::PathBuf;

use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[non_exhaustive]
#[snafu(visibility(pub))]
pub enum LoadDescriptionError {
    #[snafu(display("failed to read description file from {}: {}", path.display(), source))]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("failed to parse TOML description: {}", source))]
    ParseToml { source: toml::de::Error },

    #[snafu(display("invalid renderer configuration: {}", message))]
    InvalidRendererConfig { message: String },

    #[snafu(display("invalid camera configuration: {}", message))]
    InvalidCameraConfig { message: String },

    #[snafu(display("invalid shape definition '{}': {}", name, message))]
    InvalidShape { name: String, message: String },

    #[snafu(display("invalid material definition '{}': {}", name, message))]
    InvalidMaterial { name: String, message: String },

    #[snafu(display("invalid medium definition '{}': {}", name, message))]
    InvalidMedium { name: String, message: String },

    #[snafu(display("invalid texture definition '{}': {}", name, message))]
    InvalidTexture { name: String, message: String },

    #[snafu(display("invalid entity definition at index {}: {}", index, message))]
    InvalidEntity { index: usize, message: String },

    #[snafu(display("invalid volume definition at index {}: {}", index, message))]
    InvalidVolume { index: usize, message: String },

    #[snafu(display("material '{}' not found", name))]
    MaterialNotFound { name: String },

    #[snafu(display("medium '{}' not found", name))]
    MediumNotFound { name: String },

    #[snafu(display("texture '{}' not found", name))]
    TextureNotFound { name: String },

    #[snafu(display("failed to load external model from {}: {}", path.display(), source))]
    LoadExternalModel {
        path: PathBuf,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("missing required field: {}", field))]
    MissingField { field: String },
}
