use getset::CopyGetters;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RayScattering {
    distance: Val,
    position: Point,
}
