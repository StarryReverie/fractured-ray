use crate::domain::material::def::{DynMaterial, MaterialKind, RefDynMaterial};
use crate::domain::material::util::MaterialContainer;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::sampling::light::{AggregateLightSampler, EmptyLightSampler, LightSampling};
use crate::domain::sampling::photon::{AggregatePhotonSampler, EmptyPhotonSampler, PhotonSampling};
use crate::domain::sampling::point::{AggregatePointSampler, EmptyPointSampler, PointSampling};
use crate::domain::scene::bvh::Bvh;
use crate::domain::scene::pool::EntityPool;
use crate::domain::shape::def::Shape;
use crate::domain::shape::util::{ShapeConstructor, ShapeContainer};

use super::{EntityContainer, EntityId, EntityScene, EntitySceneBuilder};

#[derive(Debug)]
pub struct BvhEntitySceneBuilder {
    entities: Box<EntityPool>,
    light_surfaces: Vec<Box<dyn PointSampling>>,
    lights: Vec<Box<dyn LightSampling>>,
    emitters: Vec<Box<dyn PhotonSampling>>,
}

impl BvhEntitySceneBuilder {
    pub fn new() -> Self {
        Self {
            entities: Box::new(EntityPool::new()),
            light_surfaces: Vec::new(),
            lights: Vec::new(),
            emitters: Vec::new(),
        }
    }

    fn post_add_entity(&mut self, entity_id: EntityId) {
        self.register_light_surface(entity_id);
        self.register_light(entity_id);
        self.register_emitter(entity_id);
    }

    fn register_light_surface(&mut self, entity_id: EntityId) {
        if entity_id.material_id().kind() == MaterialKind::Emissive {
            let shape_id = entity_id.shape_id();
            let shape = self.entities.get_shape(shape_id).unwrap();
            if let Some(sampler) = shape.get_point_sampler(shape_id) {
                self.light_surfaces.push(sampler);
            }
        }
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
            if let RefDynMaterial::Emissive(emissive) = material {
                if let Some(sampler) = shape.get_photon_sampler(shape_id, emissive.clone()) {
                    self.emitters.push(sampler);
                }
            }
        }
    }
}

impl EntitySceneBuilder for BvhEntitySceneBuilder {
    type Output = BvhEntityScene;

    fn add<S, M>(&mut self, shape: S, material: M) -> &mut Self
    where
        S: Shape + 'static,
        M: Into<DynMaterial>,
    {
        let shape_id = self.entities.add_shape(shape);
        let material_id = self.entities.add_material(material.into());
        let entity_id = EntityId::new(shape_id, material_id);
        self.entities.register_id(entity_id);
        self.post_add_entity(entity_id);
        self
    }

    fn add_constructor<C, M>(&mut self, constructor: C, material: M) -> &mut Self
    where
        C: ShapeConstructor,
        M: Into<DynMaterial>,
    {
        let shape_ids = constructor.construct(self.entities.as_mut());
        let material_id = self.entities.add_material(material.into());

        for shape_id in shape_ids {
            let entity_id = EntityId::new(shape_id, material_id);
            self.entities.register_id(entity_id);
            self.post_add_entity(entity_id);
        }

        self
    }

    fn build(self) -> Self::Output {
        let light_surfaces: Box<dyn PointSampling> = if self.light_surfaces.len() > 1 {
            let samplers = (self.light_surfaces.into_iter())
                .map(|s| (s, Val(1.0)))
                .collect();
            Box::new(AggregatePointSampler::new(samplers))
        } else {
            (self.light_surfaces.into_iter())
                .next()
                .unwrap_or(Box::new(EmptyPointSampler::new()))
        };

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

        BvhEntityScene::new(self.entities, light_surfaces, lights, emitters)
    }
}

#[derive(Debug)]
pub struct BvhEntityScene {
    entities: Box<EntityPool>,
    bvh: Bvh<EntityId>,
    light_surfaces: Box<dyn PointSampling>,
    lights: Box<dyn LightSampling>,
    emitters: Box<dyn PhotonSampling>,
}

impl BvhEntityScene {
    fn new(
        entities: Box<EntityPool>,
        light_surfaces: Box<dyn PointSampling>,
        lights: Box<dyn LightSampling>,
        emitters: Box<dyn PhotonSampling>,
    ) -> Self {
        let ids = entities.get_ids();
        let mut bboxes = Vec::with_capacity(ids.len());
        let mut unboundeds = Vec::new();

        for id in ids {
            let sid = id.shape_id();
            match entities.get_shape(sid).unwrap().bounding_box() {
                Some(bbox) => bboxes.push((*id, bbox)),
                None => unboundeds.push(*id),
            }
        }
        let bvh = Bvh::new(bboxes, unboundeds);

        Self {
            entities,
            bvh,
            light_surfaces,
            lights,
            emitters,
        }
    }
}

impl EntityScene for BvhEntityScene {
    fn get_entities(&self) -> &dyn EntityContainer {
        &*self.entities
    }

    fn get_light_surfaces(&self) -> &dyn PointSampling {
        &*self.light_surfaces
    }

    fn get_lights(&self) -> &dyn LightSampling {
        &*self.lights
    }

    fn get_emitters(&self) -> &dyn PhotonSampling {
        &*self.emitters
    }

    fn find_intersection(&self, ray: &Ray, range: DisRange) -> Option<(RayIntersection, EntityId)> {
        self.bvh.search(ray, range, &*self.entities)
    }
}
