use getset::CopyGetters;
use rand::prelude::*;

use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::Normal;
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;

#[inline]
pub fn reflect(ray: &Ray, intersection: &RayIntersection) -> Ray {
    reflect_microfacet(ray, intersection, intersection.normal())
}

pub fn reflect_microfacet(ray: &Ray, intersection: &RayIntersection, mn: Normal) -> Ray {
    let dir_next = (ray.direction() - Val(2.0) * ray.direction().dot(mn) * mn)
        .normalize()
        .expect("reflective ray's direction should not be zero vector");
    intersection.spawn(dir_next)
}

#[inline]
pub fn pure_refract(ray: &Ray, intersection: &RayIntersection, ri: Val) -> Option<Ray> {
    pure_refract_microfacet(ray, intersection, intersection.normal(), ri)
}

pub fn pure_refract_microfacet(
    ray: &Ray,
    intersection: &RayIntersection,
    mn: Normal,
    ri: Val,
) -> Option<Ray> {
    let cos = mn.dot(-ray.direction());
    let dir_next_perp = (ray.direction() + cos * mn) / ri;

    let tmp = Val(1.0) - dir_next_perp.norm_squared();
    if tmp.is_sign_negative() {
        return None;
    }

    let dir_next_para = -tmp.sqrt() * mn;
    let dir_next = (dir_next_para + dir_next_perp)
        .normalize()
        .expect("refractive ray's direction should not be zero vector");
    Some(intersection.spawn(dir_next))
}

#[inline]
pub fn fresnel_refract(
    ray: &Ray,
    intersection: &RayIntersection,
    ri: Val,
    rng: &mut dyn RngCore,
) -> (Ray, ScatteringKind) {
    fresnel_refract_microfacet(ray, intersection, intersection.normal(), ri, rng)
}

pub fn fresnel_refract_microfacet(
    ray: &Ray,
    intersection: &RayIntersection,
    mn: Normal,
    ri: Val,
    rng: &mut dyn RngCore,
) -> (Ray, ScatteringKind) {
    let reflectance = calc_reflectance(mn.dot(-ray.direction()), ri);
    if Val(rng.random()) < reflectance {
        let ray = reflect_microfacet(ray, intersection, mn);
        (ray, ScatteringKind::new(true, reflectance))
    } else if let Some(ray) = pure_refract_microfacet(ray, intersection, mn, ri) {
        (ray, ScatteringKind::new(false, reflectance))
    } else {
        let ray = reflect_microfacet(ray, intersection, mn);
        (ray, ScatteringKind::new(true, reflectance))
    }
}

fn calc_reflectance(cos: Val, ri: Val) -> Val {
    let r0_sqrt = (Val(1.0) - ri) / (Val(1.0) + ri);
    let r0 = r0_sqrt * r0_sqrt;
    let val = r0 + (Val(1.0) - r0) * (Val(1.0) - cos).powi(5);
    val.min(Val(1.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct ScatteringKind {
    is_reflective: bool,
    reflectance: Val,
}

impl ScatteringKind {
    fn new(is_reflective: bool, reflectance: Val) -> Self {
        Self {
            is_reflective,
            reflectance,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::Vector;
    use crate::domain::math::geometry::Point;
    use crate::domain::ray::event::SurfaceSide;

    use super::*;

    #[test]
    fn reflect_succeeds() {
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
            Normal::y_direction(),
            SurfaceSide::Back,
        );

        let ray_next = reflect(&ray, &intersection);
        assert_eq!(
            ray_next.direction(),
            Vector::new(-sqrt3_2, Val(0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );
    }

    #[test]
    fn pure_refract_succeeds() {
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
            Normal::y_direction(),
            SurfaceSide::Front,
        );

        let ray_next = pure_refract(&ray, &intersection, Val(3.0).sqrt()).unwrap();
        assert_eq!(
            ray_next.direction(),
            Vector::new(Val(-0.5), -sqrt3_2, Val(0.0))
                .normalize()
                .unwrap(),
        );
    }
}
