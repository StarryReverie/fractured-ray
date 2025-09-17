pub trait Transformation: Default {
    fn inverse(self) -> Self;
}

pub trait AtomTransformation: Transformation {}

pub trait Transform<T>
where
    T: Transformation,
{
    fn transform(&self, transformation: &T) -> Self;
}
