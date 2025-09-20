use std::sync::Arc;

use smallvec::SmallVec;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::{Area, Normal, Point};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::math::transformation::Transform;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RayIntersectionPart};
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::LightSampling;
use crate::domain::sampling::photon::PhotonSampling;
use crate::domain::sampling::point::PointSampling;
use crate::domain::shape::def::{BoundingBox, Shape, ShapeKind};
use crate::domain::shape::mesh::MeshData;
use crate::domain::shape::util::ShapeId;

use super::Polygon;

#[derive(Debug, Clone)]
pub struct MeshPolygon {
    data: Arc<MeshData>,
    index: usize,
}

impl MeshPolygon {
    pub fn new(data: Arc<MeshData>, index: usize) -> Self {
        Self { data, index }
    }

    fn get_vertices(&self) -> SmallVec<[&Point; 5]> {
        let vertices = self.data.vertices();
        let polygons = self.data.polygons();
        polygons[self.index]
            .iter()
            .map(|index| &vertices[*index as usize])
            .collect::<SmallVec<[_; 5]>>()
    }

    fn to_polygon(&self) -> Polygon {
        if let Some(tr) = self.data.transformation() {
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

    fn hit_part<'a>(&self, ray: &'a Ray, range: DisRange) -> Option<RayIntersectionPart<'a>> {
        let vertices = self.get_vertices();

        assert!(vertices.len() > 3);
        let normal =
            Normal::normalize((*vertices[1] - *vertices[0]).cross(*vertices[2] - *vertices[1]))
                .expect("normal existence has been checked during mesh construction");

        if let Some(tr) = self.data.transformation() {
            let vertices_tr = (vertices.iter())
                .map(|v| v.transform(tr))
                .collect::<SmallVec<[_; 6]>>();
            let vertices_tr_ref = vertices_tr.iter().collect::<SmallVec<[_; 6]>>();
            let normal_tr = normal.transform(tr);
            Polygon::calc_ray_intersection_part(ray, range, &vertices_tr_ref, &normal_tr)
        } else {
            Polygon::calc_ray_intersection_part(ray, range, &vertices, &normal)
        }
    }

    fn complete_part(&self, part: RayIntersectionPart) -> RayIntersection {
        let vertices = self.get_vertices();

        assert!(vertices.len() > 3);
        let normal =
            Normal::normalize((*vertices[1] - *vertices[0]).cross(*vertices[2] - *vertices[1]))
                .expect("normal existence has been checked during mesh construction");

        if let Some(tr) = self.data.transformation() {
            let normal_tr = normal.transform(tr);
            Polygon::complete_ray_intersection_part(part, &normal_tr)
        } else {
            Polygon::complete_ray_intersection_part(part, &normal)
        }
    }

    fn area(&self) -> Area {
        let vertices = self.get_vertices();
        let normal = self.normal(*vertices[0]);

        let mut sum = Val(0.0);
        for i in 1..(vertices.len() - 1) {
            let side1 = *vertices[i] - *vertices[0];
            let side2 = *vertices[i + 1] - *vertices[0];
            let cross = side1.cross(side2);
            sum += cross.norm() * cross.dot(normal).signum();
        }
        Area::new(sum * Val(0.5)).unwrap()
    }

    fn normal(&self, _position: Point) -> Normal {
        let vertices = self.data.vertices();
        let polygon = &self.data.polygons()[self.index];
        assert!(polygon.len() > 3);
        let v0 = vertices[polygon[0] as usize];
        let v1 = vertices[polygon[1] as usize];
        let v2 = vertices[polygon[2] as usize];
        Normal::normalize((v1 - v0).cross(v2 - v0))
            .expect("triangle's two sides should not parallel")
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let mut vertices = self.get_vertices().into_iter();
        let init = *vertices.next().expect("init should exist");
        let (min, max) = vertices.fold((init, init), |(min, max), vertex| {
            (min.component_min(vertex), max.component_max(vertex))
        });

        match self.data.transformation() {
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

impl PartialEq for MeshPolygon {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data) && self.index == other.index
    }
}

impl Eq for MeshPolygon {}

#[cfg(test)]
mod tests {
    use crate::domain::math::numeric::Val;
    use crate::domain::shape::mesh::MeshConstructor;

    use super::*;

    #[test]
    fn mesh_bounding_box_succeeds() {
        let (_, polygons) = MeshConstructor::new(
            vec![
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(-1.0), Val(0.0)),
            ],
            vec![vec![0, 1, 2, 3]],
        )
        .unwrap()
        .construct_impl(None);

        assert_eq!(
            polygons[0].bounding_box(),
            Some(BoundingBox::new(
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
            )),
        );
    }
}
