use std::sync::Arc;

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
use crate::domain::texture::def::{UvCoordinate, UvCoordinateInterpolation};

use super::Triangle;

#[derive(Debug, Clone)]
pub struct MeshTriangle {
    data: Arc<MeshData>,
    index: usize,
}

impl MeshTriangle {
    pub fn new(data: Arc<MeshData>, index: usize) -> Self {
        Self { data, index }
    }

    fn get_vertices(&self) -> (&Point, &Point, &Point) {
        let vertices = self.data.vertices().data();
        let triangles = self.data.vertices().triangles();
        let v0 = &vertices[triangles[self.index].0 as usize];
        let v1 = &vertices[triangles[self.index].1 as usize];
        let v2 = &vertices[triangles[self.index].2 as usize];
        (v0, v1, v2)
    }

    fn get_uvs(&self) -> Option<(UvCoordinate, UvCoordinate, UvCoordinate)> {
        let uv_component = self.data.uvs()?;
        let uvs = uv_component.data();
        let triangle = &uv_component.triangles()[self.index];
        let uv0 = uvs[triangle.0 as usize];
        let uv1 = uvs[triangle.1 as usize];
        let uv2 = uvs[triangle.2 as usize];
        Some((uv0, uv1, uv2))
    }

    fn to_triangle(&self) -> Triangle {
        let (v0, v1, v2) = self.get_vertices();
        if let Some(tr) = self.data.transformation() {
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

    fn hit_part<'a>(&self, ray: &'a Ray, range: DisRange) -> Option<RayIntersectionPart<'a>> {
        let (v0, v1, v2) = self.get_vertices();
        if let Some(tr) = self.data.transformation() {
            let (v0_tr, v1_tr, v2_tr) = (v0.transform(tr), v1.transform(tr), v2.transform(tr));
            Triangle::calc_ray_intersection_part(ray, range, &v0_tr, &v1_tr, &v2_tr)
        } else {
            Triangle::calc_ray_intersection_part(ray, range, v0, v1, v2)
        }
    }

    fn complete_part(&self, part: RayIntersectionPart) -> RayIntersection {
        let (v0, v1, v2) = self.get_vertices();
        if let Some(tr) = self.data.transformation() {
            let (v0_tr, v1_tr, v2_tr) = (v0.transform(tr), v1.transform(tr), v2.transform(tr));
            let res = Triangle::complete_ray_intersection_part(part, &v0_tr, &v1_tr, &v2_tr);

            if let Some((uv0, uv1, uv2)) = self.get_uvs() {
                let uv = UvCoordinateInterpolation::new()
                    .push(v0_tr, uv0)
                    .push(v1_tr, uv1)
                    .push(v2_tr, uv2)
                    .interpolate(res.position());
                res.with_uv(uv)
            } else {
                res
            }
        } else {
            let res = Triangle::complete_ray_intersection_part(part, v0, v1, v2);

            if let Some((uv0, uv1, uv2)) = self.get_uvs() {
                let uv = UvCoordinateInterpolation::new()
                    .push(*v0, uv0)
                    .push(*v1, uv1)
                    .push(*v2, uv2)
                    .interpolate(res.position());
                res.with_uv(uv)
            } else {
                res
            }
        }
    }

    fn area(&self) -> Area {
        let (v0, v1, v2) = self.get_vertices();
        Area::new(Val(0.5) * (*v1 - *v0).cross(*v2 - *v0).norm()).unwrap()
    }

    fn normal(&self, _position: Point) -> Normal {
        let (v0, v1, v2) = self.get_vertices();
        Normal::normalize((*v1 - *v0).cross(*v2 - *v0))
            .expect("triangle's two sides should not parallel")
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let (v0, v1, v2) = self.get_vertices();
        let min = v0.component_min(v1).component_min(v2);
        let max = v0.component_max(v1).component_max(v2);

        match self.data.transformation() {
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

impl PartialEq for MeshTriangle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data) && self.index == other.index
    }
}

impl Eq for MeshTriangle {}

#[cfg(test)]
mod tests {
    use crate::domain::math::numeric::Val;
    use crate::domain::shape::mesh::MeshConstructor;

    use super::*;

    #[test]
    fn mesh_triangle_bounding_box_succeeds() {
        let (triangles, _) = MeshConstructor::new(
            vec![
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(2.0)),
            ],
            vec![vec![0, 1, 2]],
        )
        .unwrap()
        .construct_impl(None);

        assert_eq!(
            triangles[0].bounding_box(),
            Some(BoundingBox::new(
                Point::new(Val(-1.0), Val(0.0), Val(0.0)),
                Point::new(Val(1.0), Val(1.0), Val(2.0)),
            )),
        );
    }
}
