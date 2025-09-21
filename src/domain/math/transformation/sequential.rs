use getset::{CopyGetters, Getters, WithSetters};

use super::{Rotation, Scaling, Transform, Transformation, Translation};

#[derive(Debug, Default, Clone, PartialEq, Eq, Getters, CopyGetters, WithSetters)]
pub struct Sequential {
    #[getset(get = "pub", set_with = "pub")]
    scaling: Scaling,
    #[getset(get = "pub", set_with = "pub")]
    rotation: Rotation,
    #[getset(get = "pub", set_with = "pub")]
    translation: Translation,
    #[getset(get_copy = "pub")]
    inverted: bool,
}

impl Transformation for Sequential {
    fn is_identity(&self) -> bool {
        self.scaling.is_identity() && self.rotation.is_identity() && self.translation.is_identity()
    }

    fn inverse(self) -> Self {
        Self {
            scaling: self.scaling.inverse(),
            rotation: self.rotation.inverse(),
            translation: self.translation.inverse(),
            inverted: !self.inverted,
        }
    }
}

impl<T> Transform<Sequential> for T
where
    Self: Transform<Scaling>,
    Self: Transform<Rotation>,
    Self: Transform<Translation>,
{
    fn transform_impl(self, transformation: &Sequential) -> Self {
        if transformation.inverted {
            self.transform(transformation.translation())
                .transform(transformation.rotation())
                .transform(transformation.scaling())
        } else {
            self.transform(transformation.scaling())
                .transform(transformation.rotation())
                .transform(transformation.translation())
        }
    }
}
