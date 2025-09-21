use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use obj::{Group, MtlLibsLoadError, Obj, ObjData, ObjError, ObjMaterial, Object};
use snafu::prelude::*;

use crate::domain::material::def::DynMaterial;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{Val, WrappedVal};
use crate::domain::math::transformation::Transformation;
use crate::domain::scene::entity::{EntitySceneBuilder, TypedEntitySceneBuilder};
use crate::domain::shape::mesh::{MeshConstructor, MeshInstanceConstructor};
use crate::infrastructure::model::def::{
    InvalidMeshSnafu, MissingMaterialSnafu, UnspecifiedMaterialSnafu,
};

use super::obj_material::ObjMaterialConverterChain;
use super::{EntityModelLoader, EntityModelLoaderConfiguration, LoadEntityModelError};

#[derive(Debug, Clone)]
pub struct EntityObjModelLoader {
    obj: ObjData,
    path: Option<PathBuf>,
    vertices: Arc<[Point]>,
    converter: ObjMaterialConverterChain,
    material_cache: Arc<RwLock<HashMap<String, DynMaterial>>>,
}

impl EntityObjModelLoader {
    pub fn in_memory(obj: ObjData) -> Self {
        Self::new(obj, None)
    }

    pub fn parse<P>(path: P) -> Result<Self, ParseObjModelError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let mut obj = Obj::load(path).context(LoadObjSnafu { path })?;
        obj.load_mtls().context(LoadMtlSnafu { path })?;
        Ok(Self::new(obj.data, Some(path.into())))
    }

    fn new(obj: ObjData, path: Option<PathBuf>) -> Self {
        let vertices = (obj.position.iter())
            .map(Self::map_f32_array)
            .map(|[x, y, z]| Point::new(x, y, z))
            .collect::<Vec<_>>()
            .into();

        Self {
            obj,
            path,
            vertices,
            converter: ObjMaterialConverterChain::new(),
            material_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn convert_mesh(
        &self,
        object: &Object,
        group: &Group,
    ) -> Result<MeshConstructor, LoadEntityModelError> {
        let vertices = Arc::clone(&self.vertices);
        let vertex_indices = (group.polys.iter())
            .map(|poly| &poly.0)
            .map(|indices| indices.iter().map(|i| i.0).collect())
            .collect();
        let mesh = MeshConstructor::new_shared(vertices, vertex_indices).with_context(|_| {
            InvalidMeshSnafu {
                path: self.path.clone(),
                mesh_name: Self::generate_mesh_name(object, group),
            }
        })?;
        Ok(mesh)
    }

    fn convert_material(
        &self,
        object: &Object,
        group: &Group,
        materials: &HashMap<String, DynMaterial>,
    ) -> Result<DynMaterial, LoadEntityModelError> {
        let Some(material) = &group.material else {
            return UnspecifiedMaterialSnafu {
                path: self.path.clone(),
                mesh_name: Self::generate_mesh_name(object, group),
            }
            .fail();
        };
        let material = match material {
            ObjMaterial::Mtl(material) => {
                if let Some(material) = materials.get(&material.name) {
                    return Ok(material.clone());
                } else {
                    material
                }
            }
            ObjMaterial::Ref(material_name) => {
                if let Some(material) = materials.get(material_name) {
                    return Ok(material.clone());
                } else {
                    return MissingMaterialSnafu {
                        material_name: material_name.clone(),
                    }
                    .fail();
                };
            }
        };

        let name = &material.name;
        if let Some(material) = self.material_cache.read().unwrap().get(name) {
            Ok(material.clone())
        } else {
            let material = self.converter.convert(material)?;
            self.material_cache
                .write()
                .unwrap()
                .insert(name.to_owned(), material.clone());
            Ok(material)
        }
    }

    fn map_f32_array(&[x, y, z]: &[f32; 3]) -> [Val; 3] {
        let x = Val(x as WrappedVal);
        let y = Val(y as WrappedVal);
        let z = Val(z as WrappedVal);
        [x, y, z]
    }

    fn generate_mesh_name(object: &Object, group: &Group) -> String {
        format!("{}/{}", object.name, group.name)
    }
}

impl EntityModelLoader for EntityObjModelLoader {
    fn load(
        &self,
        builder: &mut dyn EntitySceneBuilder,
        config: EntityModelLoaderConfiguration,
    ) -> Result<(), LoadEntityModelError> {
        let mut meshes = Vec::with_capacity(self.obj.objects.len());
        for object in &self.obj.objects {
            for group in &object.groups {
                let mesh = self.convert_mesh(object, group)?;
                let material = self.convert_material(object, group, config.materials())?;
                meshes.push((mesh, material));
            }
        }
        if config.transformation().is_identity() {
            for (mesh, material) in meshes {
                builder.add_constructor(mesh, material);
            }
        } else {
            for (mesh, material) in meshes {
                let transformation = config.transformation().clone();
                let constructor = MeshInstanceConstructor::new(Arc::new(mesh), transformation);
                builder.add_constructor(constructor, material);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum ParseObjModelError {
    #[snafu(display("could not parse obj model `{}`", path.display()))]
    LoadObj { path: PathBuf, source: ObjError },
    #[snafu(display("could not parse mtl library `{}`", path.display()))]
    LoadMtl {
        path: PathBuf,
        source: MtlLibsLoadError,
    },
}
