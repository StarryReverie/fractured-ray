use crate::domain::math::algebra::{Product, Quaternion};
use crate::domain::math::geometry::Direction;
use crate::domain::math::numeric::Val;

use super::{AtomTransformation, Transformation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rotation {
    quaternion: Quaternion,
}

impl Rotation {
    const IDENTITY_QUATERNION: Quaternion = Quaternion::new(Val(1.0), Val(0.0), Val(0.0), Val(0.0));

    pub fn new(init_dir: Direction, final_dir: Direction, roll: Val) -> Self {
        if let Ok(axis) = Direction::normalize(init_dir.cross(final_dir)) {
            let angle = init_dir.dot(final_dir).acos();
            let rotation1 = Self::get_rotation(axis, angle);
            let quaternion = if roll == Val(0.0) {
                rotation1
            } else {
                let rotation2 = Self::get_rotation(final_dir, roll);
                rotation2 * rotation1
            };
            Self { quaternion }
        } else {
            let quaternion = Self::get_rotation(final_dir, roll);
            Self { quaternion }
        }
    }

    fn get_rotation(axis: Direction, angle: Val) -> Quaternion {
        let (sa, ca) = (Val(0.5) * angle).sin_cos();
        Quaternion::new(ca, sa * axis.x(), sa * axis.y(), sa * axis.z())
    }

    #[inline]
    pub fn quaternion(&self) -> Quaternion {
        self.quaternion
    }
}

impl Default for Rotation {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY_QUATERNION.into()
    }
}

impl From<Quaternion> for Rotation {
    #[inline]
    fn from(quaternion: Quaternion) -> Self {
        Self { quaternion }
    }
}

impl Transformation for Rotation {
    #[inline]
    fn is_identity(&self) -> bool {
        self.quaternion == Self::IDENTITY_QUATERNION
    }

    #[inline]
    fn inverse(self) -> Self {
        Self {
            quaternion: self.quaternion.conjugate(),
        }
    }
}

impl AtomTransformation for Rotation {}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::Vector;

    use super::*;

    #[test]
    fn rotation_new_succeeds() {
        let rotation = Rotation::new(
            -Direction::z_direction(),
            Direction::normalize(Vector::new(Val(-1.0), Val(1.0), Val(0.0))).unwrap(),
            Val::PI / Val(4.0),
        );

        assert_eq!(
            rotation.quaternion(),
            Quaternion::new(
                Val(0.6532814824381883),
                Val(0.2705980500730986),
                Val(0.6532814824381883),
                Val(-0.2705980500730985),
            ),
        );
    }

    #[test]
    fn rotation_inverse_succeeds() {
        let rotation = Rotation::new(
            -Direction::z_direction(),
            Direction::normalize(Vector::new(Val(-1.0), Val(1.0), Val(0.0))).unwrap(),
            Val::PI / Val(4.0),
        );

        assert_eq!(
            rotation.inverse().quaternion(),
            Quaternion::new(
                Val(0.6532814824381883),
                Val(-0.2705980500730986),
                Val(-0.6532814824381883),
                Val(0.2705980500730985),
            ),
        );
    }
}
