use std::error::Error;
use std::ops::ControlFlow;

use enum_dispatch::enum_dispatch;
use obj::Material as ExtMaterial;
use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::material::def::{DynMaterial, MaterialKind};
use crate::domain::material::primitive::*;
use crate::domain::math::geometry::SpreadAngle;
use crate::domain::math::numeric::{Val, WrappedVal};

use super::LoadEntityModelError;
use super::def::InvalidMaterialSnafu;

#[derive(Debug, Clone)]
pub struct ObjMaterialConverterChain {
    chain: Vec<DynObjMaterialConverter>,
    fallback: DynMaterial,
}

impl ObjMaterialConverterChain {
    pub fn new() -> Self {
        Self {
            chain: vec![
                EmissiveObjMaterialConverter::create(),
                DiffuseObjMaterialConverter::create(),
            ],
            fallback: Diffuse::new(Albedo::BLACK).into(),
        }
    }

    pub fn convert(&self, mtl: &ExtMaterial) -> Result<DynMaterial, LoadEntityModelError> {
        for converter in &self.chain {
            if let ControlFlow::Break(res) = converter.try_convert(mtl) {
                return res;
            }
        }
        Ok(self.fallback.clone())
    }
}

#[enum_dispatch]
trait ObjMaterialConverter: Send + Sync {
    fn try_convert(
        &self,
        mtl: &ExtMaterial,
    ) -> ControlFlow<Result<DynMaterial, LoadEntityModelError>>;
}

#[enum_dispatch(ObjMaterialConverter)]
#[derive(Debug, Clone)]
enum DynObjMaterialConverter {
    Emissive(EmissiveObjMaterialConverter),
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

def_obj_material_converter!(EmissiveObjMaterialConverter);

impl ObjMaterialConverter for EmissiveObjMaterialConverter {
    fn try_convert(
        &self,
        mtl: &ExtMaterial,
    ) -> ControlFlow<Result<DynMaterial, LoadEntityModelError>> {
        let Some([ke_r, ke_g, ke_b]) = mtl.ke.map(map_f32_array) else {
            return ControlFlow::Continue(());
        };
        wrap_break(MaterialKind::Emissive, &mtl.name, || {
            let radiance = Spectrum::new(ke_r, ke_g, ke_b);
            let emissive = Emissive::new(radiance, SpreadAngle::hemisphere());
            Ok(emissive.into())
        })
    }
}

def_obj_material_converter!(DiffuseObjMaterialConverter);

impl ObjMaterialConverter for DiffuseObjMaterialConverter {
    fn try_convert(
        &self,
        mtl: &ExtMaterial,
    ) -> ControlFlow<Result<DynMaterial, LoadEntityModelError>> {
        let Some([kd_r, kd_g, kd_b]) = mtl.kd.map(map_f32_array) else {
            return ControlFlow::Continue(());
        };
        wrap_break(MaterialKind::Diffuse, &mtl.name, || {
            let albedo = Albedo::new(kd_r, kd_g, kd_b)?;
            let diffuse = Diffuse::new(albedo);
            Ok(diffuse.into())
        })
    }
}

#[inline]
fn wrap_break<F, T, C>(
    material_kind: MaterialKind,
    material_name: &str,
    block: F,
) -> ControlFlow<Result<T, LoadEntityModelError>, C>
where
    F: FnOnce() -> Result<T, Box<dyn Error + Send + Sync>>,
{
    let res = block().with_context(|_| InvalidMaterialSnafu {
        material_kind,
        material_name,
    });
    ControlFlow::Break(res)
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
    use super::*;

    #[test]
    fn obj_material_converter_chain_convert_succeeds_given_emissive_parameters() {
        let converter = ObjMaterialConverterChain::new();

        let mtl = ExtMaterial {
            ke: Some([1.0, 2.0, 3.0]),
            ..get_initial_ext_material()
        };
        match converter.convert(&mtl).unwrap() {
            DynMaterial::Emissive(emissive) => {
                let expected_radiance = Spectrum::new(Val(1.0), Val(2.0), Val(3.0));
                assert_eq!(emissive.radiance(), expected_radiance);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn obj_material_converter_chain_convert_succeeds_given_diffuse_parameters() {
        let converter = ObjMaterialConverterChain::new();

        let mtl = ExtMaterial {
            kd: Some([0.5, 0.5, 0.5]),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Ok(DynMaterial::Diffuse(_)),
        ));

        let mtl = ExtMaterial {
            kd: Some([1.5, 1.5, 1.5]),
            ..get_initial_ext_material()
        };
        assert!(matches!(
            converter.convert(&mtl),
            Err(LoadEntityModelError::InvalidMaterial { .. }),
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
