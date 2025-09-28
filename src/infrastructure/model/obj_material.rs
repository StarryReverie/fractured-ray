use std::error::Error;
use std::sync::Arc;

use enum_dispatch::enum_dispatch;
use obj::Material as ExtMaterial;
use snafu::prelude::*;

use crate::domain::color::core::{Albedo, Spectrum};
use crate::domain::image::external::ImageRegistry;
use crate::domain::material::def::{DynMaterial, MaterialKind};
use crate::domain::material::primitive::*;
use crate::domain::math::geometry::SpreadAngle;
use crate::domain::math::numeric::{Val, WrappedVal};
use crate::domain::texture::def::{DynAlbedoTexture, DynTexture};
use crate::domain::texture::primitive::ImageMap;

use super::LoadEntityModelError;
use super::def::InvalidMaterialSnafu;

#[derive(Debug, Clone)]
pub struct ObjMaterialConverterChain {
    chain: Vec<DynObjMaterialConverter>,
    fallback: DynMaterial,
    image_registry: Arc<dyn ImageRegistry>,
}

impl ObjMaterialConverterChain {
    pub fn new(image_registry: Arc<dyn ImageRegistry>) -> Self {
        Self {
            chain: vec![
                EmissiveObjMaterialConverter::create(),
                BlurryObjMaterialConverter::create(),
                RefractiveObjMaterialConverter::create(),
                GlossyObjMaterialConverter::create(),
                SpecularObjMaterialConverter::create(),
                DiffuseObjMaterialConverter::create(),
            ],
            fallback: Diffuse::new(Albedo::BLACK).into(),
            image_registry,
        }
    }

    pub fn convert(&self, mtl: &ExtMaterial) -> Result<DynMaterial, LoadEntityModelError> {
        let mut mtl = mtl.clone();
        let materials = (self.chain.iter())
            .flat_map(|converter| converter.try_convert(&mut mtl, self.image_registry.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        match materials.len() {
            0 => Ok(self.fallback.clone()),
            1 => Ok(materials.into_iter().next().unwrap()),
            _ => {
                let res = (materials.into_iter())
                    .fold(Mixed::builder(), |builder, material| builder.add(material))
                    .build();
                match res {
                    Ok(res) => Ok(res.into()),
                    Err(err) => Err(LoadEntityModelError::InvalidMaterial {
                        material_kind: MaterialKind::Mixed,
                        material_name: mtl.name.clone(),
                        source: Box::new(err),
                    }),
                }
            }
        }
    }
}

#[enum_dispatch]
trait ObjMaterialConverter: Send + Sync {
    fn try_convert(
        &self,
        mtl: &mut ExtMaterial,
        image_registry: &dyn ImageRegistry,
    ) -> Option<Result<DynMaterial, LoadEntityModelError>>;
}

#[enum_dispatch(ObjMaterialConverter)]
#[derive(Debug, Clone)]
enum DynObjMaterialConverter {
    Emissive(EmissiveObjMaterialConverter),
    Blurry(BlurryObjMaterialConverter),
    Refractive(RefractiveObjMaterialConverter),
    Glossy(GlossyObjMaterialConverter),
    Specular(SpecularObjMaterialConverter),
    Diffuse(DiffuseObjMaterialConverter),
}

macro_rules! def_obj_material_converter {
    ($type:ident) => {
        #[derive(Debug, Clone, Default)]
        struct $type;

        impl $type {
            fn create() -> DynObjMaterialConverter {
                Self.into()
            }
        }
    };
}

macro_rules! take_all {
    ($($receiver:ident.$opt:ident),*) => {
        $(
            $receiver.$opt.take();
        )*
    };
}

def_obj_material_converter!(EmissiveObjMaterialConverter);

impl ObjMaterialConverter for EmissiveObjMaterialConverter {
    fn try_convert(
        &self,
        mtl: &mut ExtMaterial,
        image_registry: &dyn ImageRegistry,
    ) -> Option<Result<DynMaterial, LoadEntityModelError>> {
        let radiance = match get_texture(&mtl.map_ke, mtl.ke, image_registry)? {
            Ok(radiance) => radiance,
            Err(err) => return Some(Err(err)),
        };

        take_all!(mtl.ke, mtl.map_ke);
        wrap_result(MaterialKind::Emissive, &mtl.name, || {
            let emissive = Emissive::new(radiance, SpreadAngle::hemisphere());
            Ok(emissive.into())
        })
    }
}

def_obj_material_converter!(BlurryObjMaterialConverter);

impl ObjMaterialConverter for BlurryObjMaterialConverter {
    fn try_convert(
        &self,
        mtl: &mut ExtMaterial,
        image_registry: &dyn ImageRegistry,
    ) -> Option<Result<DynMaterial, LoadEntityModelError>> {
        let ni = mtl.ni.map(map_f32)?;
        let roughness = mtl.ns.map(map_f32).map(convert_ns_to_roughness)?;

        if mtl.d.is_some() {
            let albedo = match get_albedo_texture(&mtl.map_ks, mtl.ks, image_registry)? {
                Ok(albedo) => albedo,
                Err(err) => return Some(Err(err)),
            };

            take_all!(mtl.ni, mtl.ns, mtl.d, mtl.ks, mtl.map_ks);
            wrap_result(MaterialKind::Blurry, &mtl.name, || {
                let blurry = Blurry::new(albedo, ni, roughness)?;
                Ok(blurry.into())
            })
        } else if let Some([tf_r, tf_g, tf_b]) = mtl.tf.map(map_f32_array) {
            take_all!(mtl.ni, mtl.ns, mtl.tf);
            wrap_result(MaterialKind::Blurry, &mtl.name, || {
                let albedo = Albedo::new(tf_r, tf_g, tf_b)?;
                let blurry = Blurry::new(albedo, ni, roughness)?;
                Ok(blurry.into())
            })
        } else {
            None
        }
    }
}

def_obj_material_converter!(RefractiveObjMaterialConverter);

impl ObjMaterialConverter for RefractiveObjMaterialConverter {
    fn try_convert(
        &self,
        mtl: &mut ExtMaterial,
        image_registry: &dyn ImageRegistry,
    ) -> Option<Result<DynMaterial, LoadEntityModelError>> {
        let ni = mtl.ni.map(map_f32)?;

        if mtl.d.is_some() {
            let albedo = match get_albedo_texture(&mtl.map_ks, mtl.ks, image_registry)? {
                Ok(albedo) => albedo,
                Err(err) => return Some(Err(err)),
            };

            take_all!(mtl.ni, mtl.d, mtl.ks, mtl.map_ks);
            wrap_result(MaterialKind::Refractive, &mtl.name, || {
                let refractive = Refractive::new(albedo, ni)?;
                Ok(refractive.into())
            })
        } else if let Some([tf_r, tf_g, tf_b]) = mtl.tf.map(map_f32_array) {
            take_all!(mtl.ni, mtl.tf);
            wrap_result(MaterialKind::Refractive, &mtl.name, || {
                let albedo = Albedo::new(tf_r, tf_g, tf_b)?;
                let refractive = Refractive::new(albedo, ni)?;
                Ok(refractive.into())
            })
        } else {
            None
        }
    }
}

def_obj_material_converter!(GlossyObjMaterialConverter);

impl ObjMaterialConverter for GlossyObjMaterialConverter {
    fn try_convert(
        &self,
        mtl: &mut ExtMaterial,
        image_registry: &dyn ImageRegistry,
    ) -> Option<Result<DynMaterial, LoadEntityModelError>> {
        let metalness = mtl.km.map(map_f32).unwrap_or(Val(0.0));
        let roughness = mtl.ns.map(map_f32).map(convert_ns_to_roughness)?;

        let albedo = match get_albedo_texture(&mtl.map_ks, mtl.ks, image_registry)? {
            Ok(albedo) => albedo,
            Err(err) => return Some(Err(err)),
        };

        take_all!(mtl.km, mtl.ns, mtl.ks, mtl.map_ks);
        wrap_result(MaterialKind::Glossy, &mtl.name, || {
            let glossy = Glossy::new(albedo, metalness, roughness)?;
            Ok(glossy.into())
        })
    }
}

def_obj_material_converter!(SpecularObjMaterialConverter);

impl ObjMaterialConverter for SpecularObjMaterialConverter {
    fn try_convert(
        &self,
        mtl: &mut ExtMaterial,
        image_registry: &dyn ImageRegistry,
    ) -> Option<Result<DynMaterial, LoadEntityModelError>> {
        let albedo = match get_albedo_texture(&mtl.map_ks, mtl.ks, image_registry)? {
            Ok(albedo) => albedo,
            Err(err) => return Some(Err(err)),
        };

        take_all!(mtl.ks, mtl.map_ks);
        wrap_result(MaterialKind::Specular, &mtl.name, || {
            let specular = Specular::new(albedo);
            Ok(specular.into())
        })
    }
}

def_obj_material_converter!(DiffuseObjMaterialConverter);

impl ObjMaterialConverter for DiffuseObjMaterialConverter {
    fn try_convert(
        &self,
        mtl: &mut ExtMaterial,
        image_registry: &dyn ImageRegistry,
    ) -> Option<Result<DynMaterial, LoadEntityModelError>> {
        let albedo = match get_albedo_texture(&mtl.map_kd, mtl.kd, image_registry)? {
            Ok(albedo) => albedo,
            Err(err) => return Some(Err(err)),
        };

        take_all!(mtl.kd, mtl.map_kd);
        wrap_result(MaterialKind::Diffuse, &mtl.name, || {
            let diffuse = Diffuse::new(albedo);
            Ok(diffuse.into())
        })
    }
}

#[inline]
fn convert_ns_to_roughness(ns: Val) -> Val {
    (Val(1.0) - ns / Val(1000.0)).clamp(Val(0.0), Val(1.0))
}

fn get_albedo_texture(
    texture_map: &Option<String>,
    constant: Option<[f32; 3]>,
    image_registry: &dyn ImageRegistry,
) -> Option<Result<DynAlbedoTexture, LoadEntityModelError>> {
    get_texture(texture_map, constant, image_registry).map(|res| res.map(Into::into))
}

fn get_texture(
    texture_map: &Option<String>,
    constant: Option<[f32; 3]>,
    image_registry: &dyn ImageRegistry,
) -> Option<Result<DynTexture, LoadEntityModelError>> {
    let map_ks = match texture_map.as_ref().map(|name| image_registry.get(name)) {
        Some(Ok(image)) => Some(DynTexture::from(ImageMap::new(image))),
        Some(Err(err)) => {
            let err: Box<dyn Error + Send + Sync> = Box::new(err);
            return Some(Err(err).whatever_context("could not load image texture map"));
        }
        None => None,
    };

    let ks = constant.map(map_f32_array);

    match (map_ks, ks) {
        (Some(map_ks), _) => Some(Ok(map_ks)),
        (_, Some(ks)) => {
            let res = Spectrum::new(ks[0], ks[1], ks[2]);
            Some(Ok(DynTexture::from(res)))
        }
        _ => None,
    }
}

#[inline]
fn wrap_result<F, T>(
    material_kind: MaterialKind,
    material_name: &str,
    block: F,
) -> Option<Result<T, LoadEntityModelError>>
where
    F: FnOnce() -> Result<T, Box<dyn Error + Send + Sync>>,
{
    Some(block().with_context(|_| InvalidMaterialSnafu {
        material_kind,
        material_name,
    }))
}

#[inline]
fn map_f32(value: f32) -> Val {
    Val(value as WrappedVal)
}

#[inline]
fn map_f32_array(values: [f32; 3]) -> [Val; 3] {
    [map_f32(values[0]), map_f32(values[1]), map_f32(values[2])]
}

#[cfg(test)]
mod tests {
    use crate::domain::image::core::Image;
    use crate::domain::image::external::LoadImageError;

    use super::*;

    #[derive(Debug)]
    struct DummyRegistry;

    impl ImageRegistry for DummyRegistry {
        fn get(&self, _: &str) -> Result<Arc<Image>, LoadImageError> {
            whatever!("`DummyRegistry` could not load anything");
        }
    }

    #[test]
    fn obj_material_converter_chain_convert_succeeds_given_emissive_parameters() {
        let converter = ObjMaterialConverterChain::new(Arc::new(DummyRegistry));

        let mtl = ExtMaterial {
            ke: Some([1.0, 2.0, 3.0]),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Ok(DynMaterial::Emissive(_)),
        ));
    }

    #[test]
    fn obj_material_converter_chain_convert_succeeds_given_blurry_parameters() {
        let converter = ObjMaterialConverterChain::new(Arc::new(DummyRegistry));

        let mtl = ExtMaterial {
            ks: Some([0.5, 0.5, 0.5]),
            ns: Some(200.0),
            d: Some(0.8),
            ni: Some(1.5),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Ok(DynMaterial::Blurry(_)),
        ));

        let mtl = ExtMaterial {
            tf: Some([0.5, 0.5, 0.5]),
            ns: Some(200.0),
            ni: Some(1.5),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Ok(DynMaterial::Blurry(_)),
        ));
    }

    #[test]
    fn obj_material_converter_chain_convert_succeeds_given_refractive_parameters() {
        let converter = ObjMaterialConverterChain::new(Arc::new(DummyRegistry));

        let mtl = ExtMaterial {
            ks: Some([0.5, 0.5, 0.5]),
            d: Some(0.8),
            ni: Some(1.5),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Ok(DynMaterial::Refractive(_)),
        ));

        let mtl = ExtMaterial {
            tf: Some([0.5, 0.5, 0.5]),
            ni: Some(1.5),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Ok(DynMaterial::Refractive(_)),
        ));
    }

    #[test]
    fn obj_material_converter_chain_convert_succeeds_given_glossy_parameters() {
        let converter = ObjMaterialConverterChain::new(Arc::new(DummyRegistry));

        let mtl = ExtMaterial {
            ks: Some([0.5, 0.5, 0.5]),
            km: Some(0.4),
            ns: Some(800.0),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Ok(DynMaterial::Glossy(_)),
        ));
    }

    #[test]
    fn obj_material_converter_chain_convert_succeeds_given_specular_parameters() {
        let converter = ObjMaterialConverterChain::new(Arc::new(DummyRegistry));

        let mtl = ExtMaterial {
            ks: Some([0.5, 0.5, 0.5]),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Ok(DynMaterial::Specular(_)),
        ));
    }

    #[test]
    fn obj_material_converter_chain_convert_succeeds_given_diffuse_parameters() {
        let converter = ObjMaterialConverterChain::new(Arc::new(DummyRegistry));

        let mtl = ExtMaterial {
            kd: Some([0.5, 0.5, 0.5]),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Ok(DynMaterial::Diffuse(_)),
        ));
    }

    fn get_initial_ext_material() -> ExtMaterial {
        ExtMaterial {
            name: "Test.001".into(),
            ka: None,
            kd: None,
            ks: None,
            ke: None,
            km: None,
            tf: None,
            ns: None,
            ni: None,
            tr: None,
            d: None,
            illum: None,
            map_ka: None,
            map_kd: None,
            map_ks: None,
            map_ke: None,
            map_ns: None,
            map_d: None,
            map_bump: None,
            map_refl: None,
        }
    }
}
