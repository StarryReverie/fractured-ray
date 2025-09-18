use rand::prelude::*;
use rand_distr::UnitSphere;
use snafu::prelude::*;

use crate::domain::math::numeric::Val;

use super::{Product, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitVector(Vector);

impl UnitVector {
    #[inline]
    pub fn normalize(vector: Vector) -> Result<Self, TryNormalizeVectorError> {
        let norm_squared = vector.norm_squared();
        if norm_squared == Val(1.0) {
            Ok(UnitVector(vector))
        } else {
            ensure!(norm_squared > Val(0.0), ZeroVectorSnafu);
            Ok(UnitVector(vector / norm_squared.sqrt()))
        }
    }

    pub fn random(rng: &mut dyn RngCore) -> Self {
        let [x, y, z] = UnitSphere.sample(rng);
        Self(Vector::new(Val(x), Val(y), Val(z)))
    }

    pub fn random_cosine_hemisphere(normal: Self, rng: &mut dyn RngCore) -> Self {
        loop {
            let unit = Self::random(rng);
            if let Ok(direction) = (normal + unit).normalize() {
                return direction;
            }
        }
    }

    #[inline]
    pub fn x_direction() -> Self {
        Self(Vector::new(Val(1.0), Val(0.0), Val(0.0)))
    }

    #[inline]
    pub fn y_direction() -> Self {
        Self(Vector::new(Val(0.0), Val(1.0), Val(0.0)))
    }

    #[inline]
    pub fn z_direction() -> Self {
        Self(Vector::new(Val(0.0), Val(0.0), Val(1.0)))
    }

    #[inline]
    pub fn norm(&self) -> Val {
        Val(1.0)
    }

    #[inline]
    pub fn norm_squared(&self) -> Val {
        Val(1.0)
    }

    #[inline]
    pub fn to_vector(self) -> Vector {
        self.0
    }

    pub fn orthonormal_basis(&self) -> (UnitVector, UnitVector) {
        let basis1 = Vector::new(-self.0.y(), self.0.x(), Val(0.0))
            .normalize()
            .unwrap_or(UnitVector::x_direction());
        let basis2 = UnitVector(self.cross(basis1));
        (basis1, basis2)
    }
}

impl TryFrom<Vector> for UnitVector {
    type Error = TryNormalizeVectorError;

    #[inline]
    fn try_from(value: Vector) -> Result<Self, Self::Error> {
        Self::normalize(value)
    }
}

impl From<UnitVector> for Vector {
    #[inline]
    fn from(value: UnitVector) -> Self {
        value.0
    }
}

crate::impl_common_methods_for_wrapper_vector!(UnitVector);

crate::impl_add_for_wrapper_vector!(UnitVector, UnitVector);
crate::impl_add_for_wrapper_vector!(UnitVector, Vector);
crate::impl_add_for_wrapper_vector!(Vector, UnitVector);

crate::impl_sub_for_wrapper_vector!(UnitVector, UnitVector);
crate::impl_sub_for_wrapper_vector!(UnitVector, Vector);
crate::impl_sub_for_wrapper_vector!(Vector, UnitVector);

crate::impl_product_for_wrapper_vector!(UnitVector, UnitVector);
crate::impl_product_for_wrapper_vector!(UnitVector, Vector);
crate::impl_product_for_wrapper_vector!(Vector, UnitVector);

crate::impl_neg_for_wrapper_vector!(UnitVector);

crate::impl_commutative_mul_for_wrapper_vector_and_scalar!(UnitVector, Val);

crate::impl_div_for_wrapper_vector_and_scalar!(UnitVector, Val);

crate::impl_common_transformation_for_wrapper_vector!(UnitVector);

#[derive(Debug, Snafu, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNormalizeVectorError {
    #[snafu(display("couldn't convert a zero vector to a unit vector"))]
    ZeroVector,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_vector3d_linear_operations_succeed() {
        let v1 = Vector::new(Val(1.0), Val(0.0), Val(0.0))
            .normalize()
            .unwrap();
        let v2 = Vector::new(Val(0.0), Val(1.0), Val(0.0))
            .normalize()
            .unwrap();
        assert_eq!(
            v1 + v2.to_vector(),
            Vector::new(Val(1.0), Val(1.0), Val(0.0))
        );
        assert_eq!(
            v1.to_vector() - v2,
            Vector::new(Val(1.0), Val(-1.0), Val(0.0))
        );
        assert_eq!(-v1, UnitVector(Vector::new(Val(-1.0), Val(0.0), Val(0.0))));
        assert_eq!(Val(2.0) * v1, Vector::new(Val(2.0), Val(0.0), Val(0.0)));
        assert_eq!(v2 / Val(2.0), Vector::new(Val(0.0), Val(0.5), Val(0.0)));
    }

    #[test]
    fn unit_vector3d_products_succeed() {
        let v1 = Vector::new(Val(1.0), Val(0.0), Val(0.0))
            .normalize()
            .unwrap();
        let v2 = Vector::new(Val(0.0), Val(1.0), Val(0.0))
            .normalize()
            .unwrap();
        assert_eq!(v1.dot(v2), Val(0.0));
        assert_eq!(v1.cross(v2), Vector::new(Val(0.0), Val(0.0), Val(1.0)));
    }

    #[test]
    fn unit_vector3d_try_from_succeeds() {
        assert_eq!(
            Vector::new(Val(1.0), Val(2.0), Val(2.0)).normalize(),
            Ok(UnitVector(Vector::new(
                Val(1.0) / Val(3.0),
                Val(2.0) / Val(3.0),
                Val(2.0) / Val(3.0)
            ))),
        );
        assert_eq!(
            Vector::new(Val(0.0), Val(0.0), Val(0.0)).normalize(),
            Err(TryNormalizeVectorError::ZeroVector),
        );
    }
}
