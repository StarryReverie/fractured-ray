mod bsdf_ext;
mod bssrdf_ext;
mod material;

pub use bsdf_ext::{BsdfMaterialExt, FluxEstimation};
pub use bssrdf_ext::BssrdfMaterialExt;
pub use material::{
    BsdfMaterial, BssrdfMaterial, Material, MaterialContainer, MaterialId, MaterialKind,
};
