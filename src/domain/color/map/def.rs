use std::fmt::Debug;

use crate::domain::color::core::Spectrum;
use crate::domain::math::numeric::Val;

pub trait Colormap: Debug + Send + Sync {
    fn lookup(&self, value: Val) -> Spectrum;
}
