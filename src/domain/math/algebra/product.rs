use crate::domain::math::numeric::Val;

pub trait Product<Rhs = Self> {
    type Output;

    fn dot(self, rhs: Rhs) -> Val;

    fn cross(self, rhs: Rhs) -> Self::Output;
}
