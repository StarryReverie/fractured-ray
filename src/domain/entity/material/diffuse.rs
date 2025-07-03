use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::entity::shape::{DisRange, RayIntersection};
use crate::domain::geometry::{Val, Vector};
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

    fn generate_incident_ray_trace(
        &self,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> RayTrace {
        let normal = intersection.normal();
        loop {
            let (x, y, z) = rng.random::<(f64, f64, f64)>();
            let (x, y, z) = (Val(x * 2.0 - 1.0), Val(y * 2.0 - 1.0), Val(z * 2.0 - 1.0));
            if let Ok(unit) = Vector::new(x, y, z).normalize() {
                if let Ok(direction) = (normal + unit).normalize() {
                    return RayTrace::new(intersection.position(), direction);
                }
            }
        }
    }

    fn shade_impl(
        &self,
        renderer: &dyn Renderer,
        outgoing_ray_trace: RayTrace,
        depth: usize,
        incident_ray_trace: RayTrace,
    ) -> Ray {
        let incident_ray = renderer.trace(incident_ray_trace, DisRange::positive(), depth + 1);
        let color = incident_ray.color() * self.albedo;
        Ray::new(
            RayTrace::new(outgoing_ray_trace.start(), -outgoing_ray_trace.direction()),
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
        let incident_ray_trace = self.generate_incident_ray_trace(&intersection, &mut rng);
        self.shade_impl(renderer, outgoing_ray_trace, depth, incident_ray_trace)
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
        renderer.expect_trace().returning(|_, _, _| {
            Ray::new(
                RayTrace::new(
                    Point::new(Val(0.0), Val(1.0), Val(-2.0)),
                    -Vector::new(Val(1.0), Val(-2.0), Val(-2.0))
                        .normalize()
                        .unwrap(),
                ),
                Color::new(Val(0.6), Val(0.6), Val(0.6)),
            )
        });

        let outgoing_ray_trace = RayTrace::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Vector::new(Val(0.0), Val(1.0), Val(-2.0))
                .normalize()
                .unwrap(),
        );

        let incident_ray_trace = RayTrace::new(
            Point::new(Val(0.0), Val(1.0), Val(-2.0)),
            Vector::new(Val(1.0), Val(-2.0), Val(-2.0))
                .normalize()
                .unwrap(),
        );

        let ray = diffuse.shade_impl(&renderer, outgoing_ray_trace, 1, incident_ray_trace);

        let expected = Color::new(Val(0.6), Val(0.6), Val(0.6))
            * Color::new(Val(0.8), Val(0.8), Val(0.8))
            * (Val(2.0) / Val(3.0));
        assert_eq!(ray.color().red(), expected.red());
        assert_eq!(ray.color().green(), expected.green());
        assert_eq!(ray.color().blue(), expected.blue());
    }
}
