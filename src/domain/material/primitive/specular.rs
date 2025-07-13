use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::Product;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::Renderer;

#[derive(Debug, Clone, PartialEq)]
pub struct Specular {
    albedo: Color,
}

impl Specular {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }

    fn calc_reflective_ray(&self, ray: &Ray, intersection: RayIntersection) -> Ray {
        let normal = intersection.normal();
        let dir = ray.direction();
        Ray::new(
            intersection.position(),
            (dir - Val(2.0) * dir.dot(normal) * normal)
                .normalize()
                .expect("reflective ray's direction should not be zero vector"),
        )
    }
}

impl Material for Specular {
    fn material_kind(&self) -> MaterialKind {
        MaterialKind::Specular
    }

    fn shade(
        &self,
        renderer: &dyn Renderer,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        let reflective_ray = self.calc_reflective_ray(&ray, intersection);
        let color = renderer.trace(reflective_ray, DisRange::positive(), depth + 1);
        color * self.albedo
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::{UnitVector, Vector};
    use crate::domain::math::geometry::Point;
    use crate::domain::ray::SurfaceSide;

    use super::*;

    #[test]
    fn specular_calc_reflective_ray_succeeds() {
        let sqrt3_2 = Val(3.0).sqrt() / Val(2.0);

        let ray = Ray::new(
            Point::new(sqrt3_2, Val(0.5), Val(0.0)),
            Vector::new(-sqrt3_2, Val(-0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );

        let intersection = RayIntersection::new(
            Val(1.0),
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            UnitVector::y_direction(),
            SurfaceSide::Back,
        );

        let specular = Specular::new(Color::WHITE);

        let exiting_ray = specular.calc_reflective_ray(&ray, intersection);
        assert_eq!(
            exiting_ray.direction(),
            Vector::new(-sqrt3_2, Val(0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );
    }
}
