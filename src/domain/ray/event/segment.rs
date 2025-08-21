use getset::CopyGetters;

use crate::domain::math::numeric::Val;

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RaySegment {
    start: Val,
    length: Val,
}

impl RaySegment {
    pub fn new(start: Val, length: Val) -> Self {
        Self { start, length }
    }
}
