#[macro_export]
macro_rules! impl_common_methods_for_wrapper_vector {
    ($type:ty) => {
        impl $type {
            #[inline]
            pub fn x(&self) -> Val {
                self.0.x()
            }

            #[inline]
            pub fn y(&self) -> Val {
                self.0.y()
            }

            #[inline]
            pub fn z(&self) -> Val {
                self.0.z()
            }

            #[inline]
            pub fn is_perpendicular_to<V>(&self, rhs: V) -> bool
            where
                Self: Product<V, Output = $crate::domain::math::algebra::Vector>,
            {
                self.dot(rhs) == Val(0.0)
            }

            #[inline]
            pub fn is_parallel_to<V>(&self, rhs: V) -> bool
            where
                Self: Product<V, Output = $crate::domain::math::algebra::Vector>,
            {
                self.cross(rhs).is_zero()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_add_for_wrapper_vector {
    ($lhs_type:ty, $rhs_type:ty) => {
        impl std::ops::Add<$rhs_type> for $lhs_type {
            type Output = $crate::domain::math::algebra::Vector;

            #[inline]
            fn add(self, rhs: $rhs_type) -> Self::Output {
                $crate::domain::math::algebra::Vector::from(self)
                    + $crate::domain::math::algebra::Vector::from(rhs)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_sub_for_wrapper_vector {
    ($lhs_type:ty, $rhs_type:ty) => {
        impl std::ops::Sub<$rhs_type> for $lhs_type {
            type Output = $crate::domain::math::algebra::Vector;

            #[inline]
            fn sub(self, rhs: $rhs_type) -> Self::Output {
                $crate::domain::math::algebra::Vector::from(self)
                    - $crate::domain::math::algebra::Vector::from(rhs)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_product_for_wrapper_vector {
    ($lhs_type:ty, $rhs_type:ty) => {
        impl $crate::domain::math::algebra::Product<$rhs_type> for $lhs_type {
            type Output = $crate::domain::math::algebra::Vector;

            #[inline]
            fn dot(self, rhs: $rhs_type) -> Val {
                $crate::domain::math::algebra::Vector::from(self)
                    .dot($crate::domain::math::algebra::Vector::from(rhs))
            }

            #[inline]
            fn cross(self, rhs: $rhs_type) -> Self::Output {
                $crate::domain::math::algebra::Vector::from(self)
                    .cross($crate::domain::math::algebra::Vector::from(rhs))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_neg_for_wrapper_vector {
    ($type:ty) => {
        impl std::ops::Neg for $type {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self::Output {
                Self(-self.0)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_commutative_mul_for_wrapper_vector_and_scalar {
    ($lhs_type:ty, $rhs_type:ty) => {
        impl std::ops::Mul<$rhs_type> for $lhs_type {
            type Output = $crate::domain::math::algebra::Vector;

            #[inline]
            fn mul(self, rhs: $rhs_type) -> Self::Output {
                self.0 * rhs
            }
        }

        impl std::ops::Mul<$lhs_type> for $rhs_type {
            type Output = <$lhs_type as std::ops::Mul<$rhs_type>>::Output;

            #[inline]
            fn mul(self, rhs: $lhs_type) -> Self::Output {
                rhs * self
            }
        }
    };
}

#[macro_export]
macro_rules! impl_div_for_wrapper_vector_and_scalar {
    ($lhs_type:ty, $rhs_type:ty) => {
        impl std::ops::Div<$rhs_type> for $lhs_type {
            type Output = $crate::domain::math::algebra::Vector;

            #[inline]
            fn div(self, rhs: $rhs_type) -> Self::Output {
                self.0 / rhs
            }
        }
    };
}

#[macro_export]
macro_rules! impl_common_transformation_for_wrapper_vector {
    ($type:ty) => {
        impl
            $crate::domain::math::transformation::Transform<
                $crate::domain::math::transformation::Rotation,
            > for $type
        {
            #[inline]
            fn transform_impl(
                self,
                transformation: &$crate::domain::math::transformation::Rotation,
            ) -> Self {
                Self($crate::domain::math::transformation::Transform::transform(
                    self.0,
                    transformation,
                ))
            }
        }

        impl
            $crate::domain::math::transformation::Transform<
                $crate::domain::math::transformation::Scaling,
            > for $type
        {
            #[inline]
            fn transform_impl(
                self,
                _transformation: &$crate::domain::math::transformation::Scaling,
            ) -> Self {
                self
            }
        }

        impl
            $crate::domain::math::transformation::Transform<
                $crate::domain::math::transformation::Translation,
            > for $type
        {
            #[inline]
            fn transform_impl(
                self,
                transformation: &$crate::domain::math::transformation::Translation,
            ) -> Self {
                Self($crate::domain::math::transformation::Transform::transform(
                    self.0,
                    transformation,
                ))
            }
        }
    };
}
