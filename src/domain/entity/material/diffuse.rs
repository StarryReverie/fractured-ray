use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::entity::shape::{DisRange, RayIntersection};
use crate::domain::geometry::{Product, Vector};
use crate::domain::ray::{Ray, RayTrace};
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

    fn generate_ray_trace(
        &self,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> RayTrace {
        let normal = intersection.normal();
        loop {
            let (x, y, z) = rng.random::<(f64, f64, f64)>();
            let direction = Vector::new(x * 2.0 - 1.0, y * 2.0 - 1.0, z * 2.0 - 1.0);
            if direction.norm_squared() > 1e-8 && direction.dot(normal) > 0.0 {
                let direction = direction
                    .normalize()
                    .expect("direction should not be zero vector");
                return RayTrace::new(intersection.position(), direction);
            }
        }
    }

    fn shade_impl(
        &self,
        renderer: &dyn Renderer,
        ray_trace: RayTrace,
        intersection: RayIntersection,
        depth: usize,
        incident_ray_trace: RayTrace,
    ) -> Ray {
        let incident_ray = renderer.trace(incident_ray_trace, DisRange::positive(), depth + 1);
        let cos_angle = intersection.normal().dot(incident_ray.direction()).abs();
        let color = cos_angle * incident_ray.color() * self.albedo;
        Ray::new(
            RayTrace::new(ray_trace.start(), -ray_trace.direction()),
            color,
        )
    }
}

impl Material for Diffuse {
    fn shade(
        &self,
        renderer: &dyn Renderer,
        outgoing_ray_trace: RayTrace,
        intersection: RayIntersection,
        depth: usize,
    ) -> Ray {
        let mut rng = rand::rng();
        let incident_ray_trace = self.generate_ray_trace(&intersection, &mut rng);
        self.shade_impl(
            renderer,
            outgoing_ray_trace,
            intersection,
            depth,
            incident_ray_trace,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::shape::SurfaceSide;
    use crate::domain::geometry::{Point, UnitVector};
    use crate::domain::renderer::MockRenderer;

    use super::*;

    #[test]
    fn diffuse_shade_impl_succeeds() {
        let diffuse = Diffuse::new(Color::new(0.8, 0.8, 0.8));

        let mut renderer = MockRenderer::new();
        renderer.expect_trace().returning(|_, _, _| {
            Ray::new(
                RayTrace::new(
                    Point::new(0.0, 1.0, -2.0),
                    -Vector::new(1.0, -2.0, -2.0).normalize().unwrap(),
                ),
                Color::new(0.6, 0.6, 0.6),
            )
        });

        let ray_trace = RayTrace::new(
            Point::new(0.0, 0.0, 0.0),
            Vector::new(0.0, 1.0, -2.0).normalize().unwrap(),
        );

        let intersection = RayIntersection::new(
            5f64.sqrt(),
            Point::new(0.0, 1.0, -2.0),
            -UnitVector::y_direction(),
            SurfaceSide::Front,
        );

        let incident_ray_trace = RayTrace::new(
            Point::new(0.0, 1.0, -2.0),
            Vector::new(1.0, -2.0, -2.0).normalize().unwrap(),
        );

        let ray = diffuse.shade_impl(&renderer, ray_trace, intersection, 1, incident_ray_trace);

        let expected = Color::new(0.6, 0.6, 0.6) * Color::new(0.8, 0.8, 0.8) * (2.0 / 3.0);
        assert!((ray.color().red() - expected.red()).abs() < 1e-8);
        assert!((ray.color().green() - expected.green()).abs() < 1e-8);
        assert!((ray.color().blue() - expected.blue()).abs() < 1e-8);
    }
}
