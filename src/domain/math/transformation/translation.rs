use crate::domain::math::algebra::Vector;

use super::{AtomTransformation, Transformation};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Translation {
    displacement: Vector,
}

impl Translation {
    #[inline]
    pub fn new(displacement: Vector) -> Self {
        Self { displacement }
    }

    #[inline]
    pub fn displacement(&self) -> Vector {
        self.displacement
    }
}

impl Transformation for Translation {
    #[inline]
    fn is_identity(&self) -> bool {
        self.displacement.is_zero()
    }

    #[inline]
    fn inverse(self) -> Self {
        Self::new(-self.displacement)
    }
}

impl AtomTransformation for Translation {}
