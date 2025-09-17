pub trait Transformation {
    fn inverse(self) -> Self;
}

pub trait Transform<T: Transformation> {
    fn transform(&self, transformation: &T) -> Self;
}
