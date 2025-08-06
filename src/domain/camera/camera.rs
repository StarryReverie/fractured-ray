use getset::CopyGetters;
use snafu::prelude::*;

use crate::domain::math::algebra::{Product, UnitVector, Vector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;

use super::{Offset, Resolution, TryNewViewportError, Viewport};

#[derive(Debug, Clone, PartialEq, CopyGetters)]
pub struct Camera {
    #[getset(get_copy = "pub")]
    position: Point,
    #[getset(get_copy = "pub")]
    orientation: UnitVector,
    #[getset(get_copy = "pub")]
    focal_length: Val,
    viewport: Viewport,
    viewport_horizontal_edge: Vector,
    viewport_vertical_edge: Vector,
}

impl Camera {
    pub fn new(
        position: Point,
        orientation: UnitVector,
        resolution: Resolution,
        height: Val,
        focal_length: Val,
    ) -> Result<Camera, TryNewCameraError> {
        ensure!(focal_length > Val(0.0), InvalidFocalLengthSnafu);

        let viewport = Viewport::new(resolution, height).context(ViewportSnafu)?;

        let (hdir, vdir) = if orientation.x() != Val(0.0) || orientation.z() != Val(0.0) {
            let hdir = Vector::new(-orientation.z(), Val(0.0), orientation.x())
                .normalize()
                .expect("hdir shouldn't be zero vector");
            let vdir = orientation
                .cross(hdir)
                .normalize()
                .expect("vdir shouldn't be zero vector");
            (hdir, vdir)
        } else {
            let hdir = UnitVector::x_direction();
            let vdir = if orientation.y() > Val(0.0) {
                -UnitVector::z_direction()
            } else {
                UnitVector::z_direction()
            };
            (hdir, vdir)
        };

        let viewport_horizontal_edge = hdir * viewport.width();
        let viewport_vertical_edge = vdir * viewport.height();

        Ok(Self {
            position,
            orientation,
            focal_length,
            viewport,
            viewport_horizontal_edge,
            viewport_vertical_edge,
        })
    }

    pub fn resolution(&self) -> &Resolution {
        self.viewport.resolution()
    }

    pub fn calc_point_in_pixel(&self, row: usize, column: usize, offset: Offset) -> Option<Point> {
        let (vp, hp) = self.viewport.index_to_percentage(row, column, offset)?;
        let viewport_center = self.position + self.focal_length * self.orientation;
        let point = viewport_center
            + (hp - Val(0.5)) * self.viewport_horizontal_edge
            + (vp - Val(0.5)) * self.viewport_vertical_edge;
        Some(point)
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
pub enum TryNewCameraError {
    #[snafu(display("could not create a viewport"))]
    Viewport { source: TryNewViewportError },
    #[snafu(display("focal length is not positive"))]
    InvalidFocalLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_new_succeeds() {
        let camera = Camera::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            -UnitVector::z_direction(),
            Resolution::new(10, (2, 1)).unwrap(),
            Val(1.0),
            Val(1.0),
        )
        .unwrap();
        assert_eq!(
            camera.calc_point_in_pixel(0, 0, Offset::new(Val(0.0), Val(0.0)).unwrap()),
            Some(Point::new(Val(-1.0), Val(0.5), Val(-1.0))),
        );
        assert_eq!(
            camera.calc_point_in_pixel(9, 0, Offset::new(Val(1.0), Val(0.0)).unwrap()),
            Some(Point::new(Val(-1.0), Val(-0.5), Val(-1.0))),
        );
        assert_eq!(
            camera.calc_point_in_pixel(9, 19, Offset::new(Val(1.0), Val(1.0)).unwrap()),
            Some(Point::new(Val(1.0), Val(-0.5), Val(-1.0))),
        );
        assert_eq!(
            camera.calc_point_in_pixel(0, 19, Offset::new(Val(0.0), Val(1.0)).unwrap()),
            Some(Point::new(Val(1.0), Val(0.5), Val(-1.0))),
        );
    }

    #[test]
    fn camera_new_fails_when_focal_length_is_invalid() {
        assert_eq!(
            Camera::new(
                Point::new(Val(0.0), Val(2.0), Val(0.0)),
                Vector::new(Val(1.0), Val(-2.0), Val(2.0))
                    .normalize()
                    .unwrap(),
                Resolution::new(10, (2, 1)).unwrap(),
                Val(1.0),
                Val(0.0),
            ),
            Err(TryNewCameraError::InvalidFocalLength)
        );
    }

    #[test]
    fn camera_calc_point_in_pixel_succeeds() {
        let camera = Camera::new(
            Point::new(Val(0.0), Val(2.0), Val(0.0)),
            Vector::new(Val(1.0), Val(-2.0), Val(2.0))
                .normalize()
                .unwrap(),
            Resolution::new(10, (2, 1)).unwrap(),
            Val(1.0),
            Val(1.0),
        )
        .unwrap();
        assert_eq!(
            camera.calc_point_in_pixel(0, 0, Offset::center()).unwrap(),
            Point::new(
                Val(1.3172032434332408),
                Val(1.668743529958302),
                Val(0.5101419082416814)
            ),
        );
    }
}
