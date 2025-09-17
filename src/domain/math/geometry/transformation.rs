use getset::{CopyGetters, Getters, WithSetters};

use crate::domain::math::algebra::{Product, Quaternion, UnitVector, Vector};
use crate::domain::math::numeric::Val;

pub trait Transformation {
    fn inverse(self) -> Self;
}

pub trait Transform<T: Transformation> {
    fn transform(&self, transformation: &T) -> Self;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Getters, CopyGetters, WithSetters)]
pub struct Sequential {
    #[getset(get = "pub", set_with = "pub")]
    rotation: Rotation,
    #[getset(get = "pub", set_with = "pub")]
    translation: Translation,
    #[getset(get_copy = "pub")]
    inverted: bool,
}

impl Transformation for Sequential {
    fn inverse(self) -> Self {
        Self {
            rotation: self.rotation.inverse(),
            translation: self.translation.inverse(),
            inverted: !self.inverted,
        }
    }
}

impl<T> Transform<Sequential> for T
where
    Self: Transform<Rotation>,
    Self: Transform<Translation>,
{
    fn transform(&self, transformation: &Sequential) -> Self {
        if transformation.inverted {
            self.transform(&transformation.translation)
                .transform(&transformation.rotation)
        } else {
            self.transform(&transformation.rotation)
                .transform(&transformation.translation)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rotation {
    quaternion: Quaternion,
}

impl Rotation {
    pub fn new(init_dir: UnitVector, final_dir: UnitVector, roll: Val) -> Self {
        if let Ok(axis) = init_dir.cross(final_dir).normalize() {
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

    fn get_rotation(axis: UnitVector, angle: Val) -> Quaternion {
        let (sa, ca) = (Val(0.5) * angle).sin_cos();
        Quaternion::new(ca, sa * axis.x(), sa * axis.y(), sa * axis.z())
    }

    pub fn quaternion(&self) -> Quaternion {
        self.quaternion
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Quaternion::new(Val(1.0), Val(0.0), Val(0.0), Val(0.0)).into()
    }
}

impl From<Quaternion> for Rotation {
    fn from(quaternion: Quaternion) -> Self {
        Self { quaternion }
    }
}

impl Transformation for Rotation {
    fn inverse(self) -> Self {
        Self {
            quaternion: self.quaternion.conjugate(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Translation {
    displacement: Vector,
}

impl Translation {
    pub fn new(displacement: Vector) -> Self {
        Self { displacement }
    }

    pub fn displacement(&self) -> Vector {
        self.displacement
    }
}

impl Transformation for Translation {
    fn inverse(self) -> Self {
        Self::new(-self.displacement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_new_succeeds() {
        let rotation = Rotation::new(
            -UnitVector::z_direction(),
            Vector::new(Val(-1.0), Val(1.0), Val(0.0))
                .normalize()
                .unwrap(),
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
            -UnitVector::z_direction(),
            Vector::new(Val(-1.0), Val(1.0), Val(0.0))
                .normalize()
                .unwrap(),
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
