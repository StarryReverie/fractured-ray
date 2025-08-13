use crate::domain::material::def::{Material, MaterialContainer, MaterialKind};
use crate::domain::material::primitive::Emissive;
use crate::domain::math::numeric::DisRange;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::sampling::light::{AggregateLightSampler, EmptyLightSampler, LightSampling};
use crate::domain::sampling::photon::{AggregatePhotonSampler, EmptyPhotonSampler, PhotonSampling};
use crate::domain::shape::def::{Shape, ShapeConstructor, ShapeContainer, ShapeId};

use super::{Bvh, EntityContainer, EntityId, EntityPool};

pub trait Scene: Send + Sync + 'static {
    fn get_entities(&self) -> &dyn EntityContainer;

    fn get_lights(&self) -> &dyn LightSampling;

    fn get_emitters(&self) -> &dyn PhotonSampling;

    fn find_intersection(&self, ray: &Ray, range: DisRange) -> Option<(RayIntersection, EntityId)>;

    fn test_intersection(
        &self,
        ray: &Ray,
        range: DisRange,
        shape_id: ShapeId,
    ) -> Option<(RayIntersection, EntityId)> {
        if let Some((intersection, id)) = self.find_intersection(ray, range) {
            if id.shape_id() == shape_id {
                Some((intersection, id))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct BvhSceneBuilder {
    entities: Box<EntityPool>,
    ids: Vec<EntityId>,
    lights: Vec<Box<dyn LightSampling>>,
    emitters: Vec<Box<dyn PhotonSampling>>,
}

impl BvhSceneBuilder {
    pub fn new() -> Self {
        Self {
            entities: Box::new(EntityPool::new()),
            ids: Vec::new(),
            lights: Vec::new(),
            emitters: Vec::new(),
        }
    }

    pub fn add<S, M>(&mut self, shape: S, material: M) -> &mut Self
    where
        S: Shape,
        M: Material + 'static,
    {
        let shape_id = self.entities.add_shape(shape);
        let material_id = self.entities.add_material(material);
        let entity_id = EntityId::new(shape_id, material_id);
        self.ids.push(entity_id);
        self.post_add_entity(entity_id);
        self
    }

    pub fn add_constructor<C, M>(&mut self, constructor: C, material: M) -> &mut Self
    where
        C: ShapeConstructor,
        M: Material + 'static,
    {
        let shape_ids = constructor.construct(self.entities.as_mut());
        let material_id = self.entities.add_material(material);

        for shape_id in shape_ids {
            let entity_id = EntityId::new(shape_id, material_id);
            self.ids.push(entity_id);
            self.post_add_entity(entity_id);
        }

        self
    }

    fn post_add_entity(&mut self, entity_id: EntityId) {
        self.register_light(entity_id);
        self.register_emitter(entity_id);
    }

    fn register_light(&mut self, entity_id: EntityId) {
        if entity_id.material_id().kind() == MaterialKind::Emissive {
            let shape_id = entity_id.shape_id();
            let shape = self.entities.get_shape(shape_id).unwrap();
            if let Some(sampler) = shape.get_light_sampler(shape_id) {
                self.lights.push(sampler);
            }
        }
    }

    fn register_emitter(&mut self, entity_id: EntityId) {
        if entity_id.material_id().kind() == MaterialKind::Emissive {
            let shape_id = entity_id.shape_id();
            let shape = self.entities.get_shape(shape_id).unwrap();

            let material_id = entity_id.material_id();
            let material = self.entities.get_material(material_id).unwrap();
            if let Some(material) = material.as_any() {
                let emissive = material.downcast_ref::<Emissive>().unwrap();
                if let Some(sampler) = shape.get_photon_sampler(shape_id, emissive.clone()) {
                    self.emitters.push(sampler);
                }
            }
        }
    }

    pub fn build(self) -> BvhScene {
        let mut bboxes = Vec::with_capacity(self.ids.len());
        let mut unboundeds = Vec::new();

        for id in self.ids {
            let sid = id.shape_id();
            match self.entities.get_shape(sid).unwrap().bounding_box() {
                Some(bbox) => bboxes.push((id, bbox)),
                None => unboundeds.push(id),
            }
        }

        let lights: Box<dyn LightSampling> = if self.lights.len() > 1 {
            Box::new(AggregateLightSampler::new(self.lights))
        } else {
            (self.lights.into_iter())
                .next()
                .unwrap_or(Box::new(EmptyLightSampler::new()))
        };

        let emitters: Box<dyn PhotonSampling> = if self.emitters.len() > 1 {
            Box::new(AggregatePhotonSampler::new(self.emitters))
        } else {
            (self.emitters.into_iter())
                .next()
                .unwrap_or(Box::new(EmptyPhotonSampler::new()))
        };

        BvhScene {
            entities: self.entities,
            bvh: Bvh::new(bboxes),
            unboundeds,
            lights,
            emitters,
        }
    }
}

#[derive(Debug)]
pub struct BvhScene {
    entities: Box<EntityPool>,
    bvh: Bvh<EntityId>,
    unboundeds: Vec<EntityId>,
    lights: Box<dyn LightSampling>,
    emitters: Box<dyn PhotonSampling>,
}

impl BvhScene {
    fn find_intersection_with_unboundeds(
        &self,
        ray: &Ray,
        mut range: DisRange,
    ) -> Option<(RayIntersection, EntityId)> {
        let mut closet: Option<(RayIntersection, EntityId)> = None;

        for id in &self.unboundeds {
            let shape = self.entities.get_shape(id.shape_id()).unwrap();
            if let Some((closet, _)) = &closet {
                range = range.shrink_end(closet.distance());
            }
            if let Some(intersection) = shape.hit(ray, range) {
                closet = Some((intersection, *id));
            };
        }

        closet
    }
}

impl Scene for BvhScene {
    fn get_entities(&self) -> &dyn EntityContainer {
        &*self.entities
    }

    fn get_lights(&self) -> &dyn LightSampling {
        &*self.lights
    }

    fn get_emitters(&self) -> &dyn PhotonSampling {
        &*self.emitters
    }

    fn find_intersection(&self, ray: &Ray, range: DisRange) -> Option<(RayIntersection, EntityId)> {
        if let Some(mut res) = self.find_intersection_with_unboundeds(ray, range) {
            let range = range.shrink_end(res.0.distance());
            if let Some(bvh_res) = self.bvh.search(ray, range, &*self.entities) {
                res = bvh_res
            }
            Some(res)
        } else {
            self.bvh.search(ray, range, &*self.entities)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::color::Albedo;
    use crate::domain::material::primitive::Diffuse;
    use crate::domain::math::algebra::Vector;
    use crate::domain::math::geometry::Point;
    use crate::domain::math::numeric::Val;
    use crate::domain::shape::primitive::{Polygon, Sphere, Triangle};

    use super::*;

    #[test]
    fn scene_build_bvh_succeeds() {
        let mut builder = BvhSceneBuilder::new();
        builder.add(
            Sphere::new(Point::new(Val(1.0), Val(0.0), Val(2.0)), Val(1.0)).unwrap(),
            Diffuse::new(Albedo::WHITE),
        );
        builder.add(
            Triangle::new(
                Point::new(Val(-2.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(1.0)),
            )
            .unwrap(),
            Diffuse::new(Albedo::WHITE),
        );
        builder.add(
            Polygon::new([
                Point::new(Val(0.0), Val(-1.0), Val(0.0)),
                Point::new(Val(0.0), Val(-2.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(-2.0)),
                Point::new(Val(0.0), Val(0.0), Val(-1.0)),
            ])
            .unwrap(),
            Diffuse::new(Albedo::WHITE),
        );
        let scene = builder.build();

        let (intersection, _) = scene
            .find_intersection(
                &Ray::new(
                    Point::new(Val(-1.0), Val(0.0), Val(0.0)),
                    Vector::new(Val(2.0), Val(1.0), Val(2.0))
                        .normalize()
                        .unwrap(),
                ),
                DisRange::positive(),
            )
            .unwrap();
        assert_eq!(
            intersection.position(),
            Point::new(Val(-0.5), Val(0.25), Val(0.5))
        );
    }
}
