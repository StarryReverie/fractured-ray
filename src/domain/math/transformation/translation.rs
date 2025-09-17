use crate::domain::math::algebra::Vector;

use super::{AtomTransformation, Transformation};

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

impl AtomTransformation for Translation {}
