use crate::domain::material::def::{Material, MaterialContainer};
use crate::domain::math::numeric::DisRange;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeConstructor, ShapeContainer};

use super::{Bvh, EntityId, EntityPool};

pub trait Scene: Send + Sync + 'static {
    fn find_intersection(
        &self,
        ray: &Ray,
        range: DisRange,
    ) -> Option<(RayIntersection, &dyn Material)>;
}

#[derive(Debug)]
pub struct BvhSceneBuilder {
    entities: Box<EntityPool>,
    ids: Vec<EntityId>,
}

impl BvhSceneBuilder {
    pub fn new() -> Self {
        Self {
            entities: Box::new(EntityPool::new()),
            ids: Vec::new(),
        }
    }

    pub fn add<S: Shape, M: Material>(&mut self, shape: S, material: M) -> &mut Self {
        let shape_id = self.entities.add_shape(shape);
        let material_id = self.entities.add_material(material);
        self.ids.push(EntityId::new(shape_id, material_id));
        self
    }

    pub fn add_constructor<C, M>(&mut self, constructor: C, material: M) -> &mut Self
    where
        C: ShapeConstructor,
        M: Material,
    {
        let shape_ids = constructor.construct(self.entities.as_mut());
        let material_id = self.entities.add_material(material);

        for shape_id in shape_ids {
            self.ids.push(EntityId::new(shape_id, material_id));
        }

        self
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

        BvhScene {
            entities: self.entities,
            bvh: Bvh::new(bboxes),
            unboundeds,
        }
    }
}

#[derive(Debug)]
pub struct BvhScene {
    entities: Box<EntityPool>,
    bvh: Bvh,
    unboundeds: Vec<EntityId>,
}

impl BvhScene {
    fn find_intersection_with_unboundeds(
        &self,
        ray: &Ray,
        mut range: DisRange,
    ) -> Option<(RayIntersection, &dyn Material)> {
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

        if let Some((intersection, id)) = closet {
            let material = self.entities.get_material(id.material_id()).unwrap();
            Some((intersection, material))
        } else {
            None
        }
    }

    fn find_intersection_in_bvh(
        &self,
        ray: &Ray,
        range: DisRange,
    ) -> Option<(RayIntersection, &dyn Material)> {
        let res = self.bvh.search(ray, range, &*self.entities)?;
        let material = self.entities.get_material(res.1.material_id()).unwrap();
        Some((res.0, material))
    }
}

impl Scene for BvhScene {
    fn find_intersection(
        &self,
        ray: &Ray,
        range: DisRange,
    ) -> Option<(RayIntersection, &dyn Material)> {
        if let Some(mut res) = self.find_intersection_with_unboundeds(ray, range) {
            let range = range.shrink_end(res.0.distance());
            if let Some(bvh_res) = self.find_intersection_in_bvh(ray, range) {
                res = bvh_res
            }
            Some(res)
        } else {
            self.find_intersection_in_bvh(ray, range)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::color::Color;
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
            Diffuse::new(Color::WHITE),
        );
        builder.add(
            Triangle::new(
                Point::new(Val(-2.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(1.0)),
            )
            .unwrap(),
            Diffuse::new(Color::WHITE),
        );
        builder.add(
            Polygon::new([
                Point::new(Val(0.0), Val(-1.0), Val(0.0)),
                Point::new(Val(0.0), Val(-2.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(-2.0)),
                Point::new(Val(0.0), Val(0.0), Val(-1.0)),
            ])
            .unwrap(),
            Diffuse::new(Color::WHITE),
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
