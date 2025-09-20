use getset::CopyGetters;
use snafu::prelude::*;

use crate::domain::math::numeric::Val;

use super::{AtomTransformation, Transformation};

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Scaling {
    scale: Val,
}

impl Scaling {
    #[inline]
    pub fn uniform(scale: Val) -> Result<Self, TryNewScalingError> {
        ensure!(scale > Val(0.0), InvalidScaleSnafu);
        Ok(Self { scale })
    }
}

impl Default for Scaling {
    #[inline]
    fn default() -> Self {
        Self { scale: Val(1.0) }
    }
}

impl Transformation for Scaling {
    fn inverse(self) -> Self {
        Self {
            scale: self.scale.recip(),
        }
    }
}

impl AtomTransformation for Scaling {}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewScalingError {
    #[snafu(display("scale should be positive"))]
    InvalidScale,
}
