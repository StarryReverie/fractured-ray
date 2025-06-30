use crate::domain::geometry::{Point, UnitVector};

#[derive(Debug, Clone, PartialEq)]
pub struct RayTrace {
    start: Point,
    direction: UnitVector,
}

impl RayTrace {
    pub fn new(start: Point, direction: UnitVector) -> Self {
        Self { start, direction }
    }

    pub fn start(&self) -> Point {
        self.start
    }

    pub fn direction(&self) -> UnitVector {
        self.direction
    }

    pub fn at(&self, distance: f64) -> Point {
        self.start + distance * self.direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_trace_at_succeeds() {
        let ray = RayTrace::new(Point::new(0.0, 1.0, 0.0), UnitVector::x_direction());
        assert_eq!(ray.at(1.0), Point::new(1.0, 1.0, 0.0));
    }
}
