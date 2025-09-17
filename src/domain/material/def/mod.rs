mod bsdf_ext;
mod bssrdf_ext;
mod dispatch;
mod material;

pub use bsdf_ext::{BsdfMaterialExt, FluxEstimation};
pub use bssrdf_ext::BssrdfMaterialExt;
pub use dispatch::{DynMaterial, RefDynMaterial};
pub use material::{BsdfMaterial, BssrdfMaterial, Material, MaterialCategory, MaterialKind};
