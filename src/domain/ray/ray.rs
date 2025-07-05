use crate::domain::geometry::{Point, UnitVector, Val};

#[derive(Debug, Clone, PartialEq)]
pub struct Ray {
    start: Point,
    direction: UnitVector,
}

impl Ray {
    pub fn new(start: Point, direction: UnitVector) -> Self {
        Self { start, direction }
    }

    pub fn start(&self) -> Point {
        self.start
    }

    pub fn direction(&self) -> UnitVector {
        self.direction
    }

    pub fn at(&self, distance: Val) -> Point {
        self.start + distance * self.direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_at_succeeds() {
        let ray = Ray::new(
            Point::new(Val(0.0), Val(1.0), Val(0.0)),
            UnitVector::x_direction(),
        );
        assert_eq!(ray.at(Val(1.0)), Point::new(Val(1.0), Val(1.0), Val(0.0)));
    }
}
