use snafu::prelude::*;

use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UvCoordinate(Val, Val);

impl UvCoordinate {
    pub fn new(u: Val, v: Val) -> Result<Self, TryNewUvCoordinateError> {
        ensure!((Val(0.0)..=Val(1.0)).contains(&u), OutOfBoundSnafu);
        ensure!((Val(0.0)..=Val(1.0)).contains(&v), OutOfBoundSnafu);
        Ok(Self(u, v))
    }

    #[inline]
    pub fn clamp(u: Val, v: Val) -> Self {
        Self(u.clamp(Val(0.0), Val(1.0)), v.clamp(Val(0.0), Val(1.0)))
    }

    #[inline]
    pub fn u(&self) -> Val {
        self.0
    }

    #[inline]
    pub fn v(&self) -> Val {
        self.1
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewUvCoordinateError {
    #[snafu(display("UV coordinate's components should be in [0, 1]"))]
    OutOfBound,
}

#[derive(Debug, Clone)]
pub struct UvCoordinateInterpolation {
    vertices: [(Point, UvCoordinate); 3],
    len: usize,
}

impl UvCoordinateInterpolation {
    pub fn new() -> Self {
        Self {
            vertices: Default::default(),
            len: 0,
        }
    }

    pub fn push(mut self, position: Point, uv: UvCoordinate) -> Self {
        self.vertices[self.len] = (position, uv);
        self.len += 1;
        self
    }

    pub fn interpolate(&self, position: Point) -> UvCoordinate {
        assert!(
            self.vertices.len() >= 3,
            "at least three vertices are required to interpolate a UV coordinate"
        );

        let (vtx0, uv0) = self.vertices[0];
        let (vtx1, uv1) = self.vertices[1];
        let (vtx2, uv2) = self.vertices[2];

        let area0 = (vtx1 - position).cross(vtx2 - position).norm();
        let area1 = (vtx0 - position).cross(vtx2 - position).norm();
        let area2 = (vtx0 - position).cross(vtx1 - position).norm();

        let area = area0 + area1 + area2;
        let (w0, w1, w2) = (area0 / area, area1 / area, area2 / area);

        let u = (uv0.u() * w0 + uv1.u() * w1 + uv2.u() * w2).clamp(Val(0.0), Val(1.0));
        let v = (uv0.v() * w0 + uv1.v() * w1 + uv2.v() * w2).clamp(Val(0.0), Val(1.0));
        UvCoordinate(u, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_coordinate_new_fails_when_components_are_invalid() {
        assert!(matches!(
            UvCoordinate::new(Val(2.0), Val(0.0)),
            Err(TryNewUvCoordinateError::OutOfBound),
        ));
        assert!(matches!(
            UvCoordinate::new(Val(1.0), Val(-1.0)),
            Err(TryNewUvCoordinateError::OutOfBound),
        ));
    }

    #[test]
    fn uv_coordinate_interpolation_interpolate_succeeds() {
        let uv = UvCoordinateInterpolation::new()
            .push(
                Point::new(Val(0.0), Val(0.0), Val(0.0)),
                UvCoordinate::new(Val(0.0), Val(0.0)).unwrap(),
            )
            .push(
                Point::new(Val(2.0), Val(0.0), Val(0.0)),
                UvCoordinate::new(Val(1.0), Val(0.0)).unwrap(),
            )
            .push(
                Point::new(Val(0.0), Val(1.0), Val(0.0)),
                UvCoordinate::new(Val(0.0), Val(1.0)).unwrap(),
            )
            .interpolate(Point::new(Val(1.0), Val(0.5), Val(0.0)));
        assert_eq!(uv, UvCoordinate::new(Val(0.5), Val(0.5)).unwrap());
    }
}
