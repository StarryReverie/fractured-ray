pub trait Product<Rhs = Self> {
    type Output;

    fn dot(self, rhs: Rhs) -> f64;

    fn cross(self, rhs: Rhs) -> Self::Output;
}
