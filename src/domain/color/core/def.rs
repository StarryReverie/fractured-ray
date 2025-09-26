use crate::domain::math::numeric::Val;

pub trait Color
where
    Self: Send + Sync,
    Self: PartialEq + Eq + Clone + Copy,
{
    fn red(&self) -> Val;

    fn green(&self) -> Val;

    fn blue(&self) -> Val;

    fn lerp(a: Self, b: Self, t: Val) -> Self;
}
