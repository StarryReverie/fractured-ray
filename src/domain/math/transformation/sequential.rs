use getset::{CopyGetters, Getters, WithSetters};

use super::{Rotation, Transform, Transformation, Translation};

#[derive(Debug, Default, Clone, PartialEq, Eq, Getters, CopyGetters, WithSetters)]
pub struct Sequential {
    #[getset(get = "pub", set_with = "pub")]
    rotation: Rotation,
    #[getset(get = "pub", set_with = "pub")]
    translation: Translation,
    #[getset(get_copy = "pub")]
    inverted: bool,
}

impl Transformation for Sequential {
    fn inverse(self) -> Self {
        Self {
            rotation: self.rotation.inverse(),
            translation: self.translation.inverse(),
            inverted: !self.inverted,
        }
    }
}

impl<T> Transform<Sequential> for T
where
    Self: Transform<Rotation>,
    Self: Transform<Translation>,
{
    fn transform(&self, transformation: &Sequential) -> Self {
        if transformation.inverted {
            self.transform(&transformation.translation)
                .transform(&transformation.rotation)
        } else {
            self.transform(&transformation.rotation)
                .transform(&transformation.translation)
        }
    }
}
