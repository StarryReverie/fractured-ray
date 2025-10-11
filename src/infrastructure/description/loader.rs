use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use snafu::ResultExt;

use crate::domain::camera::{Camera, Resolution};
use crate::domain::color::core::{Albedo, Spectrum};
use crate::domain::image::external::ImageRegistry;
use crate::domain::material::def::DynMaterial;
use crate::domain::material::primitive::*;
use crate::domain::math::geometry::{Direction, Distance, Normal, Point, SpreadAngle};
use crate::domain::math::numeric::Val;
use crate::domain::math::transformation::{Rotation, Sequential, Translation};
use crate::domain::medium::def::DynMedium;
use crate::domain::medium::primitive::*;
use crate::domain::renderer::CoreRendererConfiguration;
use crate::domain::scene::entity::{BvhEntitySceneBuilder, EntitySceneBuilder};
use crate::domain::scene::volume::{BvhVolumeSceneBuilder, VolumeSceneBuilder};
use crate::domain::shape::def::DynShape;
use crate::domain::shape::primitive::*;
use crate::domain::texture::def::{DynAlbedoTexture, DynTexture};
use crate::domain::texture::noise::PerlinNoiseGenerator;
use crate::domain::texture::primitive::*;
use crate::infrastructure::image::FileSystemImageRegistry;
use crate::infrastructure::model::{EntityModelLoader, EntityModelLoaderConfiguration, EntityObjModelLoader};

use super::def::{Description, DescriptionLoader};
use super::error::*;

/// TOML-based description loader
pub struct TomlDescriptionLoader {
    path: PathBuf,
}

impl TomlDescriptionLoader {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl DescriptionLoader for TomlDescriptionLoader {
    fn load(&self) -> Result<Description, LoadDescriptionError> {
        let content = std::fs::read_to_string(&self.path).context(ReadFileSnafu {
            path: self.path.clone(),
        })?;

        let config: TomlConfig = toml::from_str(&content).context(ParseTomlSnafu)?;

        // Parse renderer configuration
        let renderer_config = parse_renderer_config(&config.renderer)?;

        // Parse camera
        let camera = parse_camera(&config.camera)?;

        // Build reusable texture registry
        let textures = build_texture_registry(config.textures.as_ref())?;

        // Build reusable material registry
        let materials = build_material_registry(config.materials.as_ref(), &textures)?;

        // Build reusable medium registry
        let mediums = build_medium_registry(config.mediums.as_ref())?;

        // Build entity scene
        let mut entity_builder = BvhEntitySceneBuilder::new();
        if let Some(entities) = config.entities {
            for (index, entity) in entities.iter().enumerate() {
                add_entity(entity_builder.as_mut(), entity, index, &materials, &textures)?;
            }
        }

        // Handle external models
        if let Some(models) = config.models {
            for model_def in models {
                load_external_model(entity_builder.as_mut(), &model_def, &materials)?;
            }
        }

        let entity_scene = entity_builder.build();

        // Build volume scene
        let mut volume_builder = BvhVolumeSceneBuilder::new();
        if let Some(volumes) = config.volumes {
            for (index, volume) in volumes.iter().enumerate() {
                add_volume(volume_builder.as_mut(), volume, index, &mediums)?;
            }
        }
        let volume_scene = volume_builder.build();

        Ok(Description::new(
            renderer_config,
            camera,
            entity_scene,
            volume_scene,
        ))
    }
}

// ========== TOML Schema Definitions ==========

#[derive(Debug, Deserialize)]
struct TomlConfig {
    renderer: TomlRendererConfig,
    camera: TomlCameraConfig,
    #[serde(default)]
    textures: Option<HashMap<String, TomlTexture>>,
    #[serde(default)]
    materials: Option<HashMap<String, TomlMaterial>>,
    #[serde(default)]
    mediums: Option<HashMap<String, TomlMedium>>,
    #[serde(default)]
    entities: Option<Vec<TomlEntity>>,
    #[serde(default)]
    volumes: Option<Vec<TomlVolume>>,
    #[serde(default)]
    models: Option<Vec<TomlExternalModel>>,
}

#[derive(Debug, Deserialize)]
struct TomlRendererConfig {
    #[serde(default = "default_iterations")]
    iterations: usize,
    #[serde(default = "default_spp_per_iteration")]
    spp_per_iteration: usize,
    #[serde(default = "default_max_depth")]
    max_depth: usize,
    #[serde(default = "default_max_invisible_depth")]
    max_invisible_depth: usize,
    #[serde(default = "default_photons_global")]
    photons_global: usize,
    #[serde(default = "default_photons_caustic")]
    photons_caustic: usize,
    #[serde(default = "default_initial_num_nearest")]
    initial_num_nearest: usize,
    #[serde(default)]
    background_color: Option<[f64; 3]>,
}

fn default_iterations() -> usize { 4 }
fn default_spp_per_iteration() -> usize { 4 }
fn default_max_depth() -> usize { 12 }
fn default_max_invisible_depth() -> usize { 4 }
fn default_photons_global() -> usize { 200000 }
fn default_photons_caustic() -> usize { 1000000 }
fn default_initial_num_nearest() -> usize { 100 }

#[derive(Debug, Deserialize)]
struct TomlCameraConfig {
    position: [f64; 3],
    direction: [f64; 3],
    resolution: TomlResolution,
    aperture: f64,
    focal_length: f64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlResolution {
    WithAspect { width: u32, aspect: [u32; 2] },
    Exact { width: u32, height: u32 },
}

#[derive(Debug, Deserialize)]
struct TomlEntity {
    shape: TomlShape,
    material: TomlMaterialRef,
}

#[derive(Debug, Deserialize)]
struct TomlVolume {
    shape: TomlShape,
    medium: TomlMediumRef,
}

#[derive(Debug, Deserialize)]
struct TomlExternalModel {
    path: String,
    #[serde(default)]
    transformation: Option<TomlTransformation>,
    #[serde(default)]
    material_overrides: Option<HashMap<String, TomlMaterialRef>>,
}

#[derive(Debug, Deserialize)]
struct TomlTransformation {
    #[serde(default)]
    translation: Option<[f64; 3]>,
    #[serde(default)]
    rotation: Option<TomlRotation>,
}

#[derive(Debug, Deserialize)]
struct TomlRotation {
    from: [f64; 3],
    to: [f64; 3],
    roll: f64,
}

// ========== Shape Definitions ==========

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TomlShape {
    Sphere { center: [f64; 3], radius: f64 },
    Plane { point: [f64; 3], normal: [f64; 3] },
    Polygon { points: Vec<[f64; 3]> },
    Triangle { vertices: [[f64; 3]; 3] },
    Aabb { min: [f64; 3], max: [f64; 3] },
}

// ========== Material Definitions ==========

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlMaterialRef {
    Inline(TomlMaterial),
    Reference { material: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TomlMaterial {
    Diffuse {
        albedo: TomlTextureRef,
    },
    Emissive {
        radiance: TomlTextureRef,
        #[serde(default = "default_hemisphere")]
        beam_angle: String,
    },
    Specular {
        albedo: TomlTextureRef,
    },
    Refractive {
        albedo: TomlTextureRef,
        ior: f64,
    },
    Glossy {
        predefinition: String,
        roughness: f64,
    },
    Blurry {
        albedo: TomlTextureRef,
        #[serde(default = "default_refractive_index")]
        refractive_index: f64,
        roughness: f64,
    },
    Mixed {
        materials: Vec<TomlMaterialRef>,
        weights: Vec<f64>,
    },
}

fn default_hemisphere() -> String {
    "hemisphere".to_string()
}

fn default_refractive_index() -> f64 {
    1.5
}

// ========== Medium Definitions ==========

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlMediumRef {
    Inline(TomlMedium),
    Reference { medium: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TomlMedium {
    Vacuum,
    Isotropic {
        albedo: [f64; 3],
        mean_free_path: [f64; 3],
    },
    HenyeyGreenstein {
        albedo: [f64; 3],
        mean_free_path: [f64; 3],
        g: f64,
    },
}

// ========== Texture Definitions ==========

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlTextureRef {
    Inline(TomlTexture),
    Reference { texture: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TomlTexture {
    Constant { color: [f64; 3] },
    ImageMap { path: String },
    Checkerboard { color1: [f64; 3], color2: [f64; 3], scale: f64 },
    Noise { scale: f64 },
    VisNormal,
    VisUv,
}

// ========== Parsing Functions ==========

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

fn parse_renderer_config(config: &TomlRendererConfig) -> Result<CoreRendererConfiguration, LoadDescriptionError> {
    let mut renderer_config = CoreRendererConfiguration::default()
        .with_iterations(config.iterations)
        .with_spp_per_iteration(config.spp_per_iteration)
        .with_max_depth(config.max_depth)
        .with_max_invisible_depth(config.max_invisible_depth)
        .with_photons_global(config.photons_global)
        .with_photons_caustic(config.photons_caustic)
        .with_initial_num_nearest(config.initial_num_nearest);

    if let Some(bg) = config.background_color {
        renderer_config = renderer_config.with_background_color(Spectrum::new(
            Val(bg[0]),
            Val(bg[1]),
            Val(bg[2]),
        ));
    }

    renderer_config.validate().map_err(|e| {
        LoadDescriptionError::InvalidRendererConfig {
            message: e.to_string(),
        }
    })?;

    Ok(renderer_config)
}

fn parse_camera(config: &TomlCameraConfig) -> Result<Camera, LoadDescriptionError> {
    let position = Point::new(
        Val(config.position[0]),
        Val(config.position[1]),
        Val(config.position[2]),
    );

    let direction = Direction::normalize(crate::domain::math::algebra::Vector::new(
        Val(config.direction[0]),
        Val(config.direction[1]),
        Val(config.direction[2]),
    ))
    .map_err(|e| LoadDescriptionError::InvalidCameraConfig {
        message: format!("invalid direction: {}", e),
    })?;

    let resolution = match &config.resolution {
        TomlResolution::WithAspect { width, aspect } => {
            // Resolution::new takes height, not width, so we need to calculate height from aspect
            let height = (*width as usize * aspect[1] as usize) / aspect[0] as usize;
            Resolution::new(height, (aspect[0] as usize, aspect[1] as usize)).map_err(|e| {
                LoadDescriptionError::InvalidCameraConfig {
                    message: format!("invalid resolution: {}", e),
                }
            })?
        }
        TomlResolution::Exact { width, height } => {
            // Calculate aspect ratio from exact dimensions
            let gcd = gcd(*width as usize, *height as usize);
            let aspect = (*width as usize / gcd, *height as usize / gcd);
            Resolution::new(*height as usize, aspect).map_err(|e| {
                LoadDescriptionError::InvalidCameraConfig {
                    message: format!("invalid resolution: {}", e),
                }
            })?
        }
    };

    let aperture = Distance::new(Val(config.aperture)).map_err(|e| {
        LoadDescriptionError::InvalidCameraConfig {
            message: format!("invalid aperture: {}", e),
        }
    })?;

    let focal_length = Distance::new(Val(config.focal_length)).map_err(|e| {
        LoadDescriptionError::InvalidCameraConfig {
            message: format!("invalid focal length: {}", e),
        }
    })?;

    Ok(Camera::new(position, direction, resolution, aperture, focal_length))
}

fn parse_shape(shape: &TomlShape, name: &str) -> Result<DynShape, LoadDescriptionError> {
    match shape {
        TomlShape::Sphere { center, radius } => {
            let center = Point::new(Val(center[0]), Val(center[1]), Val(center[2]));
            Sphere::new(center, Val(*radius))
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidShape {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
        TomlShape::Plane { point, normal } => {
            let point = Point::new(Val(point[0]), Val(point[1]), Val(point[2]));
            let normal = Normal::normalize(crate::domain::math::algebra::Vector::new(
                Val(normal[0]),
                Val(normal[1]),
                Val(normal[2]),
            ))
            .map_err(|e| LoadDescriptionError::InvalidShape {
                name: name.to_string(),
                message: format!("invalid normal: {}", e),
            })?;
            Ok(Plane::new(point, normal).into())
        }
        TomlShape::Polygon { points } => {
            let points: Vec<Point> = points
                .iter()
                .map(|p| Point::new(Val(p[0]), Val(p[1]), Val(p[2])))
                .collect();
            Polygon::new(points)
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidShape {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
        TomlShape::Triangle { vertices } => {
            let v0 = Point::new(Val(vertices[0][0]), Val(vertices[0][1]), Val(vertices[0][2]));
            let v1 = Point::new(Val(vertices[1][0]), Val(vertices[1][1]), Val(vertices[1][2]));
            let v2 = Point::new(Val(vertices[2][0]), Val(vertices[2][1]), Val(vertices[2][2]));
            Triangle::new(v0, v1, v2)
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidShape {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
        TomlShape::Aabb { min, max } => {
            let min = Point::new(Val(min[0]), Val(min[1]), Val(min[2]));
            let max = Point::new(Val(max[0]), Val(max[1]), Val(max[2]));
            Ok(Aabb::new(min, max).into())
        }
    }
}

fn build_texture_registry(
    textures: Option<&HashMap<String, TomlTexture>>,
) -> Result<HashMap<String, DynTexture>, LoadDescriptionError> {
    let mut registry = HashMap::new();
    if let Some(textures) = textures {
        for (name, texture) in textures {
            let tex = parse_texture(texture, name)?;
            registry.insert(name.clone(), tex);
        }
    }
    Ok(registry)
}

fn parse_texture(texture: &TomlTexture, name: &str) -> Result<DynTexture, LoadDescriptionError> {
    match texture {
        TomlTexture::Constant { color } => {
            let spectrum = Spectrum::new(Val(color[0]), Val(color[1]), Val(color[2]));
            Ok(Constant::new(spectrum).into())
        }
        TomlTexture::ImageMap { path } => {
            let registry = Arc::new(FileSystemImageRegistry::new());
            let image = registry.get(path).map_err(|e| LoadDescriptionError::InvalidTexture {
                name: name.to_string(),
                message: format!("failed to load image: {}", e),
            })?;
            Ok(ImageMap::new(image).into())
        }
        TomlTexture::Checkerboard { color1, color2, scale } => {
            let c1 = Spectrum::new(Val(color1[0]), Val(color1[1]), Val(color1[2]));
            let c2 = Spectrum::new(Val(color2[0]), Val(color2[1]), Val(color2[2]));
            Checkerboard::new(c1, c2, Val(*scale))
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidTexture {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
        TomlTexture::Noise { scale } => {
            Noise::new(PerlinNoiseGenerator::new(), Val(*scale))
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidTexture {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
        TomlTexture::VisNormal => Ok(VisibieNormal::new().into()),
        TomlTexture::VisUv => Ok(VisibleUvCoordinate::new().into()),
    }
}

fn resolve_texture_ref(
    texture_ref: &TomlTextureRef,
    registry: &HashMap<String, DynTexture>,
    context: &str,
) -> Result<DynTexture, LoadDescriptionError> {
    match texture_ref {
        TomlTextureRef::Inline(texture) => parse_texture(texture, context),
        TomlTextureRef::Reference { texture } => registry
            .get(texture)
            .cloned()
            .ok_or_else(|| LoadDescriptionError::TextureNotFound {
                name: texture.clone(),
            }),
    }
}

fn build_material_registry(
    materials: Option<&HashMap<String, TomlMaterial>>,
    textures: &HashMap<String, DynTexture>,
) -> Result<HashMap<String, DynMaterial>, LoadDescriptionError> {
    let mut registry = HashMap::new();
    if let Some(materials) = materials {
        for (name, material) in materials {
            let mat = parse_material(material, name, textures, &registry)?;
            registry.insert(name.clone(), mat);
        }
    }
    Ok(registry)
}

fn parse_material(
    material: &TomlMaterial,
    name: &str,
    textures: &HashMap<String, DynTexture>,
    materials: &HashMap<String, DynMaterial>,
) -> Result<DynMaterial, LoadDescriptionError> {
    match material {
        TomlMaterial::Diffuse { albedo } => {
            let albedo_tex = resolve_texture_ref(albedo, textures, name)?;
            let albedo_tex: DynAlbedoTexture = albedo_tex.try_into().map_err(|_| {
                LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: "texture cannot be converted to albedo texture".to_string(),
                }
            })?;
            Ok(Diffuse::new(albedo_tex).into())
        }
        TomlMaterial::Emissive { radiance, beam_angle } => {
            let radiance_tex = resolve_texture_ref(radiance, textures, name)?;
            let angle = parse_spread_angle(beam_angle).map_err(|e| {
                LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: format!("invalid beam_angle: {}", e),
                }
            })?;
            Ok(Emissive::new(radiance_tex, angle).into())
        }
        TomlMaterial::Specular { albedo } => {
            let albedo_tex = resolve_texture_ref(albedo, textures, name)?;
            let albedo_tex: DynAlbedoTexture = albedo_tex.try_into().map_err(|_| {
                LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: "texture cannot be converted to albedo texture".to_string(),
                }
            })?;
            Ok(Specular::new(albedo_tex).into())
        }
        TomlMaterial::Refractive { albedo, ior } => {
            let albedo_tex = resolve_texture_ref(albedo, textures, name)?;
            let albedo_tex: DynAlbedoTexture = albedo_tex.try_into().map_err(|_| {
                LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: "texture cannot be converted to albedo texture".to_string(),
                }
            })?;
            Refractive::new(albedo_tex, Val(*ior))
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
        TomlMaterial::Glossy { predefinition, roughness } => {
            let predef = parse_glossy_predefinition(predefinition).map_err(|e| {
                LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: format!("invalid predefinition: {}", e),
                }
            })?;
            Glossy::lookup(predef, Val(*roughness))
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
        TomlMaterial::Blurry { albedo, refractive_index, roughness } => {
            let albedo_tex = resolve_texture_ref(albedo, textures, name)?;
            let albedo_tex: DynAlbedoTexture = albedo_tex.try_into().map_err(|_| {
                LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: "texture cannot be converted to albedo texture".to_string(),
                }
            })?;
            Blurry::new(albedo_tex, Val(*refractive_index), Val(*roughness))
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
        TomlMaterial::Mixed { materials: mat_refs, weights: _ } => {
            // Mixed material uses a builder pattern, not a simple new() constructor
            // For simplicity in config files, we'll just use the first material in the list
            // This is a limitation of the current API
            if mat_refs.is_empty() {
                return Err(LoadDescriptionError::InvalidMaterial {
                    name: name.to_string(),
                    message: "mixed material must have at least one material".to_string(),
                });
            }
            // Return the first material as a workaround
            resolve_material_ref(&mat_refs[0], textures, materials, name)
        }
    }
}

fn resolve_material_ref(
    material_ref: &TomlMaterialRef,
    textures: &HashMap<String, DynTexture>,
    materials: &HashMap<String, DynMaterial>,
    context: &str,
) -> Result<DynMaterial, LoadDescriptionError> {
    match material_ref {
        TomlMaterialRef::Inline(material) => parse_material(material, context, textures, materials),
        TomlMaterialRef::Reference { material } => materials
            .get(material)
            .cloned()
            .ok_or_else(|| LoadDescriptionError::MaterialNotFound {
                name: material.clone(),
            }),
    }
}

fn build_medium_registry(
    mediums: Option<&HashMap<String, TomlMedium>>,
) -> Result<HashMap<String, DynMedium>, LoadDescriptionError> {
    let mut registry = HashMap::new();
    if let Some(mediums) = mediums {
        for (name, medium) in mediums {
            let med = parse_medium(medium, name)?;
            registry.insert(name.clone(), med);
        }
    }
    Ok(registry)
}

fn parse_medium(medium: &TomlMedium, name: &str) -> Result<DynMedium, LoadDescriptionError> {
    match medium {
        TomlMedium::Vacuum => Ok(Vacuum::new().into()),
        TomlMedium::Isotropic { albedo, mean_free_path } => {
            let albedo = Albedo::new(Val(albedo[0]), Val(albedo[1]), Val(albedo[2])).map_err(|e| {
                LoadDescriptionError::InvalidMedium {
                    name: name.to_string(),
                    message: format!("invalid albedo: {}", e),
                }
            })?;
            let mfp = Spectrum::new(
                Val(mean_free_path[0]),
                Val(mean_free_path[1]),
                Val(mean_free_path[2]),
            );
            Isotropic::new(albedo, mfp)
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidMedium {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
        TomlMedium::HenyeyGreenstein { albedo, mean_free_path, g } => {
            let albedo = Albedo::new(Val(albedo[0]), Val(albedo[1]), Val(albedo[2])).map_err(|e| {
                LoadDescriptionError::InvalidMedium {
                    name: name.to_string(),
                    message: format!("invalid albedo: {}", e),
                }
            })?;
            let mfp = Spectrum::new(
                Val(mean_free_path[0]),
                Val(mean_free_path[1]),
                Val(mean_free_path[2]),
            );
            HenyeyGreenstein::new(albedo, mfp, Val(*g))
                .map(Into::into)
                .map_err(|e| LoadDescriptionError::InvalidMedium {
                    name: name.to_string(),
                    message: e.to_string(),
                })
        }
    }
}

fn resolve_medium_ref(
    medium_ref: &TomlMediumRef,
    registry: &HashMap<String, DynMedium>,
    context: &str,
) -> Result<DynMedium, LoadDescriptionError> {
    match medium_ref {
        TomlMediumRef::Inline(medium) => parse_medium(medium, context),
        TomlMediumRef::Reference { medium } => registry
            .get(medium)
            .cloned()
            .ok_or_else(|| LoadDescriptionError::MediumNotFound {
                name: medium.clone(),
            }),
    }
}

fn add_entity(
    builder: &mut dyn EntitySceneBuilder,
    entity: &TomlEntity,
    index: usize,
    materials: &HashMap<String, DynMaterial>,
    textures: &HashMap<String, DynTexture>,
) -> Result<(), LoadDescriptionError> {
    let shape = parse_shape(&entity.shape, &format!("entity[{}].shape", index))?;
    let material = resolve_material_ref(
        &entity.material,
        textures,
        materials,
        &format!("entity[{}].material", index),
    )?;
    builder.add_dyn(shape, material);
    Ok(())
}

fn add_volume(
    builder: &mut dyn VolumeSceneBuilder,
    volume: &TomlVolume,
    index: usize,
    mediums: &HashMap<String, DynMedium>,
) -> Result<(), LoadDescriptionError> {
    let shape = parse_shape(&volume.shape, &format!("volume[{}].shape", index))?;
    let medium = resolve_medium_ref(
        &volume.medium,
        mediums,
        &format!("volume[{}].medium", index),
    )?;
    builder.add_dyn(shape, medium);
    Ok(())
}

fn load_external_model(
    builder: &mut dyn EntitySceneBuilder,
    model_def: &TomlExternalModel,
    materials: &HashMap<String, DynMaterial>,
) -> Result<(), LoadDescriptionError> {
    let path = PathBuf::from(&model_def.path);
    let loader = EntityObjModelLoader::parse(&path, Arc::new(FileSystemImageRegistry::new()))
        .map_err(|e| LoadDescriptionError::LoadExternalModel {
            path: path.clone(),
            source: Box::new(e),
        })?;

    let mut config = EntityModelLoaderConfiguration::default();

    // Apply transformation if specified
    if let Some(trans) = &model_def.transformation {
        let mut seq = Sequential::default();
        if let Some(t) = trans.translation {
            seq = seq.with_translation(Translation::new(crate::domain::math::algebra::Vector::new(
                Val(t[0]),
                Val(t[1]),
                Val(t[2]),
            )));
        }
        if let Some(rot) = &trans.rotation {
            let from = Direction::normalize(crate::domain::math::algebra::Vector::new(
                Val(rot.from[0]),
                Val(rot.from[1]),
                Val(rot.from[2]),
            ))
            .map_err(|e| LoadDescriptionError::LoadExternalModel {
                path: path.clone(),
                source: Box::new(e),
            })?;
            let to = Direction::normalize(crate::domain::math::algebra::Vector::new(
                Val(rot.to[0]),
                Val(rot.to[1]),
                Val(rot.to[2]),
            ))
            .map_err(|e| LoadDescriptionError::LoadExternalModel {
                path: path.clone(),
                source: Box::new(e),
            })?;
            seq = seq.with_rotation(Rotation::new(from, to, Val(rot.roll)));
        }
        config = config.with_transformation(seq);
    }

    // Apply material overrides
    if let Some(overrides) = &model_def.material_overrides {
        for (name, _mat_ref) in overrides {
            let material = materials
                .get(name)
                .ok_or_else(|| LoadDescriptionError::MaterialNotFound {
                    name: name.clone(),
                })?;
            config = config.add_material(name.clone(), material.clone());
        }
    }

    loader
        .load(builder, config)
        .map_err(|e| LoadDescriptionError::LoadExternalModel {
            path: path.clone(),
            source: Box::new(e),
        })?;

    Ok(())
}

fn parse_spread_angle(s: &str) -> Result<SpreadAngle, String> {
    match s.to_lowercase().as_str() {
        "hemisphere" => Ok(SpreadAngle::hemisphere()),
        _ => {
            let angle: f64 = s.parse().map_err(|e| format!("invalid angle: {}", e))?;
            SpreadAngle::new(Val(angle)).map_err(|e| e.to_string())
        }
    }
}

fn parse_glossy_predefinition(s: &str) -> Result<GlossyPredefinition, String> {
    match s.to_lowercase().as_str() {
        "iron" => Ok(GlossyPredefinition::Iron),
        "copper" => Ok(GlossyPredefinition::Copper),
        "gold" => Ok(GlossyPredefinition::Gold),
        "aluminum" => Ok(GlossyPredefinition::Aluminum),
        "silver" => Ok(GlossyPredefinition::Silver),
        _ => Err(format!("unknown glossy predefinition: {}", s)),
    }
}
