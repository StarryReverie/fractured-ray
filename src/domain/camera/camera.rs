use getset::CopyGetters;

use crate::domain::math::algebra::{Product, Vector};
use crate::domain::math::geometry::{Direction, Distance, Point};
use crate::domain::math::numeric::Val;

use super::{Offset, Resolution, Viewport};

#[derive(Debug, Clone, PartialEq, CopyGetters)]
pub struct Camera {
    #[getset(get_copy = "pub")]
    position: Point,
    #[getset(get_copy = "pub")]
    orientation: Direction,
    #[getset(get_copy = "pub")]
    focal_length: Distance,
    viewport: Viewport,
    viewport_horizontal_edge: Vector,
    viewport_vertical_edge: Vector,
}

impl Camera {
    pub fn new(
        position: Point,
        orientation: Direction,
        resolution: Resolution,
        height: Distance,
        focal_length: Distance,
    ) -> Camera {
        let viewport = Viewport::new(resolution, height);

        let (hdir, vdir) = if orientation.x() != Val(0.0) || orientation.z() != Val(0.0) {
            let hdir =
                Direction::normalize(Vector::new(-orientation.z(), Val(0.0), orientation.x()))
                    .expect("hdir shouldn't be zero vector");
            let vdir = Direction::normalize(orientation.cross(hdir))
                .expect("vdir shouldn't be zero vector");
            (hdir, vdir)
        } else {
            let hdir = Direction::x_direction();
            let vdir = if orientation.y() > Val(0.0) {
                -Direction::z_direction()
            } else {
                Direction::z_direction()
            };
            (hdir, vdir)
        };

        let viewport_horizontal_edge = hdir * viewport.width().value();
        let viewport_vertical_edge = vdir * viewport.height().value();

        Self {
            position,
            orientation,
            focal_length,
            viewport,
            viewport_horizontal_edge,
            viewport_vertical_edge,
        }
    }

    pub fn resolution(&self) -> &Resolution {
        self.viewport.resolution()
    }

    pub fn calc_point_in_pixel(&self, row: usize, column: usize, offset: Offset) -> Option<Point> {
        let (vp, hp) = self.viewport.index_to_percentage(row, column, offset)?;
        let viewport_center = self.position + self.focal_length.value() * self.orientation;
        let point = viewport_center
            + (hp - Val(0.5)) * self.viewport_horizontal_edge
            + (vp - Val(0.5)) * self.viewport_vertical_edge;
        Some(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_new_succeeds() {
        let camera = Camera::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            -Direction::z_direction(),
            Resolution::new(10, (2, 1)).unwrap(),
            Distance::new(Val(1.0)).unwrap(),
            Distance::new(Val(1.0)).unwrap(),
        );

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
    fn camera_calc_point_in_pixel_succeeds() {
        let camera = Camera::new(
            Point::new(Val(0.0), Val(2.0), Val(0.0)),
            Direction::normalize(Vector::new(Val(1.0), Val(-2.0), Val(2.0))).unwrap(),
            Resolution::new(10, (2, 1)).unwrap(),
            Distance::new(Val(1.0)).unwrap(),
            Distance::new(Val(1.0)).unwrap(),
        );

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
