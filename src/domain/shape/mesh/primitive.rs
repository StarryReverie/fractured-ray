use std::sync::Arc;

use smallvec::SmallVec;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::geometry::{Point, Transform};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::LightSampling;
use crate::domain::sampling::photon::PhotonSampling;
use crate::domain::sampling::point::PointSampling;
use crate::domain::shape::def::{BoundingBox, Shape, ShapeKind};
use crate::domain::shape::primitive::{Polygon, Triangle};
use crate::domain::shape::util::ShapeId;

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

    fn to_triangle(&self) -> Triangle {
        let (v0, v1, v2) = self.get_vertices();
        if let Some(tr) = &self.data.transformation {
            Triangle::new(v0.transform(tr), v1.transform(tr), v2.transform(tr)).unwrap()
        } else {
            Triangle::new(*v0, *v1, *v2).unwrap()
        }
    }
}

impl Shape for MeshTriangle {
    fn kind(&self) -> ShapeKind {
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

    fn area(&self) -> Val {
        let (v0, v1, v2) = self.get_vertices();
        Val(0.5) * (*v1 - *v0).cross(*v2 - *v0).norm()
    }

    fn normal(&self, _position: Point) -> UnitVector {
        let (v0, v1, v2) = self.get_vertices();
        (*v1 - *v0)
            .cross(*v2 - *v0)
            .normalize()
            .expect("triangle's two sides should not parallel")
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
}

impl Sampleable for MeshTriangle {
    fn get_point_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        self.to_triangle().get_point_sampler(shape_id)
    }

    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        self.to_triangle().get_light_sampler(shape_id)
    }

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        self.to_triangle().get_photon_sampler(shape_id, emissive)
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

    fn to_polygon(&self) -> Polygon {
        if let Some(tr) = &self.data.transformation {
            let vertices = self.get_vertices().into_iter().map(|v| v.transform(tr));
            Polygon::new(vertices).unwrap()
        } else {
            let vertices = self.get_vertices().into_iter().cloned();
            Polygon::new(vertices).unwrap()
        }
    }
}

impl Shape for MeshPolygon {
    fn kind(&self) -> ShapeKind {
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

    fn area(&self) -> Val {
        let vertices = self.get_vertices();
        let normal = self.normal(*vertices[0]);

        let mut sum = Val(0.0);
        for i in 1..(vertices.len() - 1) {
            let side1 = *vertices[i] - *vertices[0];
            let side2 = *vertices[i + 1] - *vertices[0];
            let cross = side1.cross(side2);
            sum += cross.norm() * cross.dot(normal).signum();
        }
        sum * Val(0.5)
    }

    fn normal(&self, _position: Point) -> UnitVector {
        let vertices = &self.data.vertices;
        let polygon = &self.data.polygons[self.index];
        assert!(polygon.len() > 3);
        let v0 = vertices[polygon[0] as usize];
        let v1 = vertices[polygon[1] as usize];
        let v2 = vertices[polygon[2] as usize];
        (v1 - v0)
            .cross(v2 - v0)
            .normalize()
            .expect("triangle's two sides should not parallel")
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
}

impl Sampleable for MeshPolygon {
    fn get_point_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        self.to_polygon().get_point_sampler(shape_id)
    }

    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        self.to_polygon().get_light_sampler(shape_id)
    }

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        self.to_polygon().get_photon_sampler(shape_id, emissive)
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
