use std::sync::Arc;

use smallvec::SmallVec;

use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::{Point, Transform};
use crate::domain::math::numeric::DisRange;
use crate::domain::ray::sampling::LightSampling;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{BoundingBox, Shape, ShapeId, ShapeKind};
use crate::domain::shape::primitive::{Polygon, Triangle};

use super::MeshData;

#[derive(Debug, Clone)]
pub struct MeshTriangle {
    pub(super) data: Arc<MeshData>,
    pub(super) index: usize,
}

impl MeshTriangle {
    fn get_vertices(&self) -> (&Point, &Point, &Point) {
        let vertices = &self.data.vertices;
        let triangles = &self.data.triangles;
        let v0 = &vertices[triangles[self.index].0 as usize];
        let v1 = &vertices[triangles[self.index].1 as usize];
        let v2 = &vertices[triangles[self.index].2 as usize];
        (v0, v1, v2)
    }
}

impl Shape for MeshTriangle {
    fn shape_kind(&self) -> ShapeKind {
        ShapeKind::MeshTriangle
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        let (v0, v1, v2) = self.get_vertices();
        let tr = &self.data.transformation;
        let inv_tr = &self.data.inv_transformation;

        match tr.as_ref().zip(inv_tr.as_ref()) {
            None => Triangle::calc_ray_intersection(ray, range, v0, v1, v2),
            Some((tr, inv_tr)) => {
                let ray = ray.transform(inv_tr);
                let res = Triangle::calc_ray_intersection(&ray, range, v0, v1, v2)?;
                Some(res.transform(tr))
            }
        }
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let (v0, v1, v2) = self.get_vertices();
        let min = v0.component_min(v1).component_min(v2);
        let max = v0.component_max(v1).component_max(v2);

        match &self.data.transformation {
            None => Some(BoundingBox::new(min, max)),
            Some(tr) => Some(BoundingBox::new(min, max).transform(tr)),
        }
    }

    fn get_sampler(&self, _shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct MeshPolygon {
    pub(super) data: Arc<MeshData>,
    pub(super) index: usize,
}

impl MeshPolygon {
    fn get_vertices(&self) -> SmallVec<[&Point; 5]> {
        let vertices = &self.data.vertices;
        let polygons = &self.data.polygons;
        polygons[self.index]
            .iter()
            .map(|index| &vertices[*index as usize])
            .collect::<SmallVec<[_; 5]>>()
    }
}

impl Shape for MeshPolygon {
    fn shape_kind(&self) -> ShapeKind {
        ShapeKind::MeshPolygon
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        let vertices = self.get_vertices();

        assert!(vertices.len() > 3);
        let normal = (*vertices[1] - *vertices[0])
            .cross(*vertices[2] - *vertices[1])
            .normalize()
            .expect("normal existence has been checked during mesh construction");

        let tr = &self.data.transformation;
        let inv_tr = &self.data.inv_transformation;

        match tr.as_ref().zip(inv_tr.as_ref()) {
            None => Polygon::calc_ray_intersection(ray, range, &vertices, &normal),
            Some((tr, inv_tr)) => {
                let ray = ray.transform(inv_tr);
                let res = Polygon::calc_ray_intersection(&ray, range, &vertices, &normal)?;
                Some(res.transform(tr))
            }
        }
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let mut vertices = self.get_vertices().into_iter();
        let init = *vertices.next().expect("init should exist");
        let (min, max) = vertices.fold((init, init), |(min, max), vertex| {
            (min.component_min(vertex), max.component_max(vertex))
        });

        match &self.data.transformation {
            None => Some(BoundingBox::new(min, max)),
            Some(tr) => Some(BoundingBox::new(min, max).transform(tr)),
        }
    }

    fn get_sampler(&self, _shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::numeric::Val;
    use crate::domain::shape::mesh::MeshConstructor;

    use super::*;

    #[test]
    fn mesh_bounding_box_succeeds() {
        let (triangles, polygons) = MeshConstructor::new(
            vec![
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(2.0)),
            ],
            vec![vec![0, 1, 2, 3], vec![0, 1, 4]],
        )
        .unwrap()
        .construct_impl(None, None);

        assert_eq!(
            triangles[0].bounding_box(),
            Some(BoundingBox::new(
                Point::new(Val(-1.0), Val(0.0), Val(0.0)),
                Point::new(Val(1.0), Val(1.0), Val(2.0)),
            )),
        );

        assert_eq!(
            polygons[0].bounding_box(),
            Some(BoundingBox::new(
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
            )),
        );
    }
}
