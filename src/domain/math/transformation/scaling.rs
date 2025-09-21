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
    const IDENTITY_SCALE: Val = Val(1.0);

    #[inline]
    pub fn uniform(scale: Val) -> Result<Self, TryNewScalingError> {
        ensure!(scale > Val(0.0), InvalidScaleSnafu);
        Ok(Self { scale })
    }
}

impl Default for Scaling {
    #[inline]
    fn default() -> Self {
        Self {
            scale: Self::IDENTITY_SCALE,
        }
    }
}

impl Transformation for Scaling {
    #[inline]
    fn is_identity(&self) -> bool {
        self.scale == Self::IDENTITY_SCALE
    }

    #[inline]
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
