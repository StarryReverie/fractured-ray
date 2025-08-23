use std::ops::Bound;

use getset::{CopyGetters, Getters};

use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::scene::entity::EntityScene;
use crate::domain::shape::def::ShapeId;

pub struct VisibilityTester<'s, 'r> {
    scene: &'s dyn EntityScene,
    ray_next: &'r Ray,
}

impl<'s, 'r> VisibilityTester<'s, 'r> {
    pub fn new(scene: &'s dyn EntityScene, ray_next: &'r Ray) -> Self {
        Self { scene, ray_next }
    }

    pub fn test(&self, distance: Val, target_id: ShapeId) -> Option<LightTarget<'s>> {
        let scene = &self.scene;

        let range = (Bound::Excluded(Val(0.0)), Bound::Included(distance));
        let range = DisRange::from(range);

        let res = scene.test_intersection(self.ray_next, range, target_id);
        if let Some((intersection_next, id)) = res {
            let id = id.material_id();
            let material = scene.get_entities().get_material(id).unwrap();
            if material.kind() == MaterialKind::Emissive {
                Some(LightTarget::new(intersection_next, material))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn cast(&self) -> Option<LightTarget<'s>> {
        let scene = &self.scene;
        let range = DisRange::positive();

        let res = scene.find_intersection(self.ray_next, range);
        if let Some((intersection_next, id)) = res {
            let id = id.material_id();
            let material = scene.get_entities().get_material(id).unwrap();
            if material.kind() == MaterialKind::Emissive {
                Some(LightTarget::new(intersection_next, material))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Getters, CopyGetters)]
pub struct LightTarget<'a> {
    #[getset(get = "pub")]
    intersection: RayIntersection,
    #[getset(get_copy = "pub")]
    light: &'a dyn Material,
}

impl<'a> LightTarget<'a> {
    fn new(intersection: RayIntersection, light: &'a dyn Material) -> Self {
        Self {
            intersection,
            light,
        }
    }

    pub fn as_some(&self) -> Option<(&RayIntersection, &dyn Material)> {
        Some((&self.intersection, self.light))
    }
}

impl<'a> From<LightTarget<'a>> for (RayIntersection, &'a dyn Material) {
    fn from(value: LightTarget<'a>) -> Self {
        (value.intersection, value.light)
    }
}
