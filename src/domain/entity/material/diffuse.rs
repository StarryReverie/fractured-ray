use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::entity::shape::{DisRange, RayIntersection};
use crate::domain::geometry::{Val, Vector, WrappedVal};
use crate::domain::ray::Ray;
use crate::domain::renderer::Renderer;

use super::Material;

#[derive(Debug, Clone, PartialEq)]
pub struct Diffuse {
    albedo: Color,
}

impl Diffuse {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }

    fn generate_reflective_ray(
        &self,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Ray {
        let normal = intersection.normal();
        loop {
            let (x, y, z) = rng.random::<(WrappedVal, WrappedVal, WrappedVal)>();
            let (x, y, z) = (Val(x * 2.0 - 1.0), Val(y * 2.0 - 1.0), Val(z * 2.0 - 1.0));
            if let Ok(unit) = Vector::new(x, y, z).normalize() {
                if let Ok(direction) = (normal + unit).normalize() {
                    return Ray::new(intersection.position(), direction);
                }
            }
        }
    }

    fn shade_impl(&self, renderer: &dyn Renderer, ray: Ray, depth: usize) -> Color {
        let color = renderer.trace(ray, DisRange::positive(), depth + 1);
        color * self.albedo
    }
}

impl Material for Diffuse {
    fn shade(
        &self,
        renderer: &dyn Renderer,
        _ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        let mut rng = rand::rng();
        let reflective_ray = self.generate_reflective_ray(&intersection, &mut rng);
        self.shade_impl(renderer, reflective_ray, depth)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::geometry::Point;
    use crate::domain::renderer::MockRenderer;

    use super::*;

    #[test]
    fn diffuse_shade_impl_succeeds() {
        let diffuse = Diffuse::new(Color::new(Val(0.8), Val(0.8), Val(0.8)));

        let mut renderer = MockRenderer::new();
        renderer
            .expect_trace()
            .returning(|_, _, _| Color::new(Val(0.6), Val(0.6), Val(0.6)));

        let ray = Ray::new(
            Point::new(Val(0.0), Val(1.0), Val(-2.0)),
            Vector::new(Val(1.0), Val(-2.0), Val(-2.0))
                .normalize()
                .unwrap(),
        );

        let color = diffuse.shade_impl(&renderer, ray, 1);

        let expected =
            Color::new(Val(0.6), Val(0.6), Val(0.6)) * Color::new(Val(0.8), Val(0.8), Val(0.8));
        assert_eq!(color.red(), expected.red());
        assert_eq!(color.green(), expected.green());
        assert_eq!(color.blue(), expected.blue());
    }
}
