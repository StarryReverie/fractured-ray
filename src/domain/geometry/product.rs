pub trait Product<Rhs = Self> {
    type Output;

    fn dot(self, rhs: Rhs) -> f32;

    fn cross(self, rhs: Rhs) -> Self::Output;
}
