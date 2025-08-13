use getset::{CopyGetters, Getters};

use crate::domain::math::algebra::{Product, UnitVector, Vector};

use super::Point;

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Frame {
    tangent: UnitVector,
    cross: UnitVector,
    normal: UnitVector,
}

impl Frame {
    #[inline]
    pub fn new(normal: UnitVector) -> Self {
        let (tangent, cross) = normal.orthonormal_basis();
        Self {
            tangent,
            cross,
            normal,
        }
    }

    #[inline]
    pub fn to_canonical(&self, coord: Vector) -> Vector {
        coord.x() * self.tangent + coord.y() * self.cross + coord.z() * self.normal
    }

    #[inline]
    pub fn to_canonical_unit(&self, coord: UnitVector) -> UnitVector {
        self.to_canonical(coord.into()).normalize().unwrap()
    }

    #[inline]
    pub fn to_local(&self, coord: Vector) -> Vector {
        Vector::new(
            coord.dot(self.tangent),
            coord.dot(self.cross),
            coord.dot(self.normal),
        )
    }

    #[inline]
    pub fn to_local_unit(&self, coord: UnitVector) -> UnitVector {
        self.to_local(coord.into()).normalize().unwrap()
    }

    #[inline]
    pub fn permute_axes(self) -> Self {
        Self {
            tangent: self.normal,
            cross: self.tangent,
            normal: self.cross,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct PositionedFrame {
    #[getset(get_copy = "pub")]
    origin: Point,
    #[getset(get = "pub")]
    unpositioned: Frame,
}

impl PositionedFrame {
    #[inline]
    pub fn new(origin: Point, normal: UnitVector) -> Self {
        let unpositioned = Frame::new(normal);
        Self {
            origin,
            unpositioned,
        }
    }

    #[inline]
    pub fn tangent(&self) -> UnitVector {
        self.unpositioned.tangent()
    }

    #[inline]
    pub fn cross(&self) -> UnitVector {
        self.unpositioned.cross()
    }

    #[inline]
    pub fn normal(&self) -> UnitVector {
        self.unpositioned.normal()
    }

    #[inline]
    pub fn to_canonical(&self, coord: Point) -> Point {
        self.origin + self.unpositioned.to_canonical(coord.into())
    }

    #[inline]
    pub fn to_local(&self, coord: Point) -> Point {
        self.unpositioned.to_local(coord - self.origin).into()
    }

    #[inline]
    pub fn permute_axes(self) -> Self {
        Self {
            unpositioned: self.unpositioned.permute_axes(),
            ..self
        }
    }
}
