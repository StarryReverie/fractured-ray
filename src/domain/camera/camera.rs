use snafu::prelude::*;

use crate::domain::geometry::{Point, Product, UnitVector, Vector};

use super::{Offset, Resolution, TryNewViewportError, Viewport};

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    position: Point,
    orientation: UnitVector,
    focal_length: f32,
    viewport: Viewport,
    viewport_horizontal_edge: Vector,
    viewport_vertical_edge: Vector,
}

impl Camera {
    pub fn new(
        position: Point,
        orientation: UnitVector,
        resolution: Resolution,
        height: f32,
        focal_length: f32,
    ) -> Result<Camera, TryNewCameraError> {
        ensure!(focal_length > 0.0, InvalidFocalLengthSnafu);

        let viewport = Viewport::new(resolution, height).context(ViewportSnafu)?;

        let (hdir, vdir) = if orientation.x() != 0.0 || orientation.z() != 0.0 {
            let hdir = Vector::new(-orientation.z(), 0.0, orientation.x())
                .normalize()
                .expect("hdir shouldn't be zero vector");
            let vdir = orientation
                .cross(hdir)
                .normalize()
                .expect("vdir shouldn't be zero vector");
            (hdir, vdir)
        } else {
            let hdir = UnitVector::x_direction();
            let vdir = if orientation.y() > 0.0 {
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

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn orientation(&self) -> UnitVector {
        self.orientation
    }

    pub fn focal_length(&self) -> f32 {
        self.focal_length
    }

    pub fn resolution(&self) -> &Resolution {
        self.viewport.resolution()
    }

    pub fn calc_point_in_pixel(&self, row: usize, column: usize, offset: Offset) -> Option<Point> {
        let (vp, hp) = self.viewport.index_to_percentage(row, column, offset)?;
        let viewport_center = self.position + self.focal_length * self.orientation;
        let point = viewport_center
            + (hp - 0.5) * self.viewport_horizontal_edge
            + (vp - 0.5) * self.viewport_vertical_edge;
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
            Point::new(0.0, 0.0, 0.0),
            -UnitVector::z_direction(),
            Resolution::new(10, (2, 1)).unwrap(),
            1.0,
            1.0,
        )
        .unwrap();
        assert_eq!(
            camera.calc_point_in_pixel(0, 0, Offset::new(0.0, 0.0).unwrap()),
            Some(Point::new(-1.0, 0.5, -1.0)),
        );
        assert_eq!(
            camera.calc_point_in_pixel(9, 0, Offset::new(1.0, 0.0).unwrap()),
            Some(Point::new(-1.0, -0.5, -1.0)),
        );
        assert_eq!(
            camera.calc_point_in_pixel(9, 19, Offset::new(1.0, 1.0).unwrap()),
            Some(Point::new(1.0, -0.5, -1.0)),
        );
        assert_eq!(
            camera.calc_point_in_pixel(0, 19, Offset::new(0.0, 1.0).unwrap()),
            Some(Point::new(1.0, 0.5, -1.0)),
        );
    }

    #[test]
    fn camera_new_fails_when_focal_length_is_invalid() {
        assert_eq!(
            Camera::new(
                Point::new(0.0, 2.0, 0.0),
                Vector::new(1.0, -2.0, 2.0).normalize().unwrap(),
                Resolution::new(10, (2, 1)).unwrap(),
                1.0,
                0.0,
            ),
            Err(TryNewCameraError::InvalidFocalLength)
        );
    }

    #[test]
    fn camera_calc_point_in_pixel_succeeds() {
        let camera = Camera::new(
            Point::new(0.0, 2.0, 0.0),
            Vector::new(1.0, -2.0, 2.0).normalize().unwrap(),
            Resolution::new(10, (2, 1)).unwrap(),
            1.0,
            1.0,
        )
        .unwrap();
        assert_eq!(
            camera.calc_point_in_pixel(0, 0, Offset::center()),
            Some(Point::new(1.3172033, 1.6687435, 0.51014197))
        );
    }
}
