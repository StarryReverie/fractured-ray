pub trait Transformation: Default {
    fn is_identity(&self) -> bool;

    fn inverse(self) -> Self;
}

pub trait AtomTransformation: Transformation {}

pub trait Transform<T>: Sized
where
    T: Transformation,
{
    fn transform(self, transformation: &T) -> Self {
        if transformation.is_identity() {
            self
        } else {
            self.transform_impl(transformation)
        }
    }

    fn transform_impl(self, transformation: &T) -> Self;
}
