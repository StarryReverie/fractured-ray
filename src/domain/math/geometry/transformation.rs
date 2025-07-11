use crate::domain::math::algebra::{Quaternion, Vector};
use crate::domain::math::numeric::Val;

pub trait Transformation {
    fn inverse(self) -> Self;
}

pub trait Transform<T: Transformation> {
    fn transform(&self, transformation: &T) -> Self;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AllTransformation {
    pub rotation: Rotation,
    pub translation: Translation,
    pub inverse: bool,
}

impl Transformation for AllTransformation {
    fn inverse(self) -> Self {
        Self {
            rotation: self.rotation.inverse(),
            translation: self.translation.inverse(),
            inverse: !self.inverse,
        }
    }
}

impl<T> Transform<AllTransformation> for T
where
    Self: Transform<Rotation>,
    Self: Transform<Translation>,
{
    fn transform(&self, transformation: &AllTransformation) -> Self {
        if transformation.inverse {
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
    pub fn new(yaw: Val, pitch: Val, roll: Val) -> Self {
        Quaternion::euler(yaw, pitch, roll).into()
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
