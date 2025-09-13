mod dispatch;
mod homogeneous_ext;
mod medium;

pub use dispatch::{DynMedium, RefDynMedium};
pub use homogeneous_ext::HomogeneousMediumExt;
pub use medium::{HomogeneousMedium, Medium, MediumKind};
