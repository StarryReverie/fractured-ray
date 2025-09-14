use smallvec::SmallVec;
use snafu::prelude::*;
use spade::{DelaunayTriangulation, Point2, Triangulation};

use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::geometry::{Point, Rotation, Transform, Transformation};
use crate::domain::math::numeric::{DisRange, Val, WrappedVal};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::{LightSamplerAdapter, LightSampling};
use crate::domain::sampling::photon::{PhotonSamplerAdapter, PhotonSampling};
use crate::domain::sampling::point::{PointSampling, PolygonPointSampler};
use crate::domain::shape::def::{BoundingBox, Shape, ShapeKind};
use crate::domain::shape::util::ShapeId;

use super::{Plane, Triangle, TryNewTriangleError};

#[derive(Debug, Clone, PartialEq)]
pub struct Polygon(PolygonInner);

#[derive(Debug, Clone, PartialEq)]
pub enum PolygonInner {
    Triangle(Triangle),
    General {
        vertices: Vec<Point>,
        normal: UnitVector,
    },
}

impl Polygon {
    pub fn new<I>(vertices: I) -> Result<Self, TryNewPolygonError>
    where
        I: IntoIterator<Item = Point>,
    {
        let vertices = vertices.into_iter().collect::<Vec<_>>();
        ensure!(vertices.len() >= 3, TooFewVerticesSnafu);

        if vertices.len() == 3 {
            Self::new_triangular_polygon(vertices)
        } else {
            Self::new_general_polygon(vertices)
        }
    }

    fn new_triangular_polygon(vertices: Vec<Point>) -> Result<Polygon, TryNewPolygonError> {
        assert!(vertices.len() == 3);

        let [vertex0, vertex1, vertex2] = vertices[..] else {
            unreachable!("vectices has exactly 3 elements");
        };

        match Triangle::new(vertex0, vertex1, vertex2) {
            Ok(triangle) => Ok(Self(PolygonInner::Triangle(triangle))),
            Err(TryNewTriangleError::DuplicatedVertices) => {
                Err(TryNewPolygonError::DuplicatedVertices)
            }
            Err(TryNewTriangleError::ParallelSides) => {
                Err(TryNewPolygonError::ParallelAdjacentSides)
            }
        }
    }

    fn new_general_polygon(vertices: Vec<Point>) -> Result<Polygon, TryNewPolygonError> {
        assert!(vertices.len() > 3);

        let no_duplicated = vertices
            .iter()
            .enumerate()
            .all(|(i, p)| vertices.iter().take_while(|q| *q != p).count() == i);
        ensure!(no_duplicated, DuplicatedVerticesSnafu);

        let sides = (vertices.iter().skip(1))
            .chain(vertices.iter().take(1))
            .zip(vertices.iter())
            .map(|(next, prev)| *next - *prev)
            .collect::<Vec<_>>();

        let no_parallel = (sides.iter().skip(1))
            .chain(sides.iter().take(1))
            .zip(sides.iter())
            .all(|(next, prev)| !prev.is_parallel_to(*next));
        ensure!(no_parallel, ParallelAdjacentSidesSnafu);

        let normal = sides[0]
            .cross(sides[1])
            .normalize()
            .expect("side[0] and side[1] should not be zero vectors and should not be parallel");
        let is_flat = sides.iter().all(|s| s.is_perpendicular_to(normal));
        ensure!(is_flat, NotFlatSnafu);

        Ok(Self(PolygonInner::General { vertices, normal }))
    }

    pub fn calc_ray_intersection(
        ray: &Ray,
        range: DisRange,
        vertices: &[&Point],
        normal: &UnitVector,
    ) -> Option<RayIntersection> {
        assert!(vertices.len() > 3);
        let intersection = Plane::calc_ray_intersection(ray, range, vertices[0], normal)?;
        if Self::is_intersection_inside_polygon(&intersection, vertices) {
            Some(intersection)
        } else {
            None
        }
    }

    fn is_intersection_inside_polygon(intersection: &RayIntersection, vertices: &[&Point]) -> bool {
        let projection_axis = Self::select_projection_axis(intersection.normal());
        let to_vertices = Self::project_vectors(intersection, vertices, projection_axis);
        Self::calc_angle_sum(to_vertices) != Val(0.0)
    }

    fn select_projection_axis(normal: UnitVector) -> usize {
        let max_component = (normal.x().abs())
            .max(normal.y().abs())
            .max(normal.z().abs());
        if max_component == normal.x().abs() {
            0
        } else if max_component == normal.y().abs() {
            1
        } else {
            2
        }
    }

    fn project_vectors(
        intersection: &RayIntersection,
        vertices: &[&Point],
        projection_axis: usize,
    ) -> Vec<(Val, Val)> {
        vertices
            .iter()
            .map(|v| {
                let p = intersection.position();
                match projection_axis {
                    0 => (v.y() - p.y(), v.z() - p.z()),
                    1 => (v.x() - p.x(), v.z() - p.z()),
                    2 => (v.x() - p.x(), v.y() - p.y()),
                    _ => unreachable!("projection_axis should only be 0, 1 or 2"),
                }
            })
            .collect::<Vec<_>>()
    }

    fn calc_angle_sum(to_vertices: Vec<(Val, Val)>) -> Val {
        (to_vertices.iter().skip(1))
            .chain(to_vertices.iter().take(1))
            .zip(to_vertices.iter())
            .map(|(&(x1, y1), &(x2, y2))| {
                let (dot, cross) = (x1 * x2 + y1 * y2, x1 * y2 - x2 * y1);
                let (ns1, ns2) = (x1 * x1 + y1 * y1, x2 * x2 + y2 * y2);
                let angle = (dot / (ns1 * ns2).sqrt()).acos();
                angle * cross.signum()
            })
            .sum::<Val>()
    }

    pub fn triangulate(&self) -> Vec<Triangle> {
        match &self.0 {
            PolygonInner::Triangle(triangle) => vec![triangle.clone(); 1],
            PolygonInner::General { vertices, normal } => {
                assert!(vertices.len() >= 3);

                let tr = Rotation::new(*normal, UnitVector::z_direction(), Val(0.0));
                let z = vertices[0].transform(&tr).z();
                let vertices_2d = vertices
                    .iter()
                    .map(|v| v.transform(&tr))
                    .map(|v| Point2::new(v.x().0, v.y().0))
                    .collect();
                let triangulation =
                    DelaunayTriangulation::<Point2<WrappedVal>>::bulk_load_stable(vertices_2d)
                        .unwrap();

                let inv_tr = tr.inverse();
                let mut triangles = Vec::with_capacity(triangulation.num_inner_faces());
                for face in triangulation.inner_faces() {
                    let [v0, v1, v2] = face.vertices().map(|vertex| {
                        let pos = vertex.position();
                        Point::new(Val(pos.x), Val(pos.y), z).transform(&inv_tr)
                    });
                    let triangle_normal = (v1 - v0).cross(v2 - v0);
                    if triangle_normal.dot(*normal) > Val(0.0) {
                        triangles.push(Triangle::new(v0, v1, v2).unwrap());
                    } else {
                        triangles.push(Triangle::new(v0, v2, v1).unwrap());
                    }
                }
                triangles
            }
        }
    }
}

impl Shape for Polygon {
    fn kind(&self) -> ShapeKind {
        ShapeKind::Polygon
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        match &self.0 {
            PolygonInner::Triangle(triangle) => triangle.hit(ray, range),
            PolygonInner::General { vertices, normal } => {
                let vertices = vertices.iter().collect::<SmallVec<[&Point; 6]>>();
                Self::calc_ray_intersection(ray, range, &vertices, normal)
            }
        }
    }

    fn area(&self) -> Val {
        match &self.0 {
            PolygonInner::Triangle(triangle) => triangle.area(),
            PolygonInner::General { vertices, normal } => {
                let mut sum = Val(0.0);
                for i in 1..(vertices.len() - 1) {
                    let side1 = vertices[i] - vertices[0];
                    let side2 = vertices[i + 1] - vertices[0];
                    let cross = side1.cross(side2);
                    sum += cross.norm() * cross.dot(*normal).signum();
                }
                sum * Val(0.5)
            }
        }
    }

    fn normal(&self, position: Point) -> UnitVector {
        match &self.0 {
            PolygonInner::Triangle(triangle) => triangle.normal(position),
            PolygonInner::General { normal, .. } => *normal,
        }
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        match &self.0 {
            PolygonInner::Triangle(triangle) => triangle.bounding_box(),
            PolygonInner::General { vertices, .. } => {
                let init = vertices[0];
                let (min, max) = vertices.iter().fold((init, init), |(min, max), vertex| {
                    (min.component_min(vertex), max.component_max(vertex))
                });
                Some(BoundingBox::new(min, max))
            }
        }
    }
}

impl Sampleable for Polygon {
    fn get_point_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        Some(Box::new(PolygonPointSampler::new(shape_id, self.clone())))
    }

    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        match &self.0 {
            PolygonInner::Triangle(triangle) => triangle.get_light_sampler(shape_id),
            PolygonInner::General { .. } => {
                let inner = PolygonPointSampler::new(shape_id, self.clone());
                let sampler = LightSamplerAdapter::new(inner);
                Some(Box::new(sampler))
            }
        }
    }

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        match &self.0 {
            PolygonInner::Triangle(triangle) => triangle.get_photon_sampler(shape_id, emissive),
            PolygonInner::General { .. } => {
                let inner = PolygonPointSampler::new(shape_id, self.clone());
                let sampler = PhotonSamplerAdapter::new(inner, emissive);
                Some(Box::new(sampler))
            }
        }
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewPolygonError {
    #[snafu(display("polygon requires at least 3 vertices"))]
    TooFewVertices,
    #[snafu(display("polygon has duplicated vertices"))]
    DuplicatedVertices,
    #[snafu(display("polygon has parallel adjacent sides"))]
    ParallelAdjacentSides,
    #[snafu(display("polygon is not a flat shape"))]
    NotFlat,
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::Vector;
    use crate::domain::ray::event::SurfaceSide;

    use super::*;

    #[test]
    fn polygon_new_succeeds() {
        assert!(
            Polygon::new([
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(2.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(3.0)),
            ])
            .is_ok()
        );
        assert!(
            Polygon::new([
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(2.0), Val(1.0)),
                Point::new(Val(-1.0), Val(1.0), Val(3.0)),
                Point::new(Val(0.0), Val(-1.0), Val(2.0)),
            ])
            .is_ok()
        );
    }

    #[test]
    fn polygon_new_fails_when_vertices_are_too_few() {
        assert!(matches!(
            Polygon::new([
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
            ]),
            Err(TryNewPolygonError::TooFewVertices),
        ));
    }

    #[test]
    fn polygon_new_fails_when_vertices_are_duplicated() {
        assert!(matches!(
            Polygon::new([
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(3.0)),
                Point::new(Val(0.0), Val(-1.0), Val(2.0)),
            ]),
            Err(TryNewPolygonError::DuplicatedVertices),
        ));
    }

    #[test]
    fn polygon_new_fails_when_adjacent_sides_are_parallel() {
        assert!(matches!(
            Polygon::new([
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(2.0), Val(0.0)),
                Point::new(Val(0.0), Val(1.0), Val(1.5)),
                Point::new(Val(0.0), Val(0.0), Val(3.0)),
            ]),
            Err(TryNewPolygonError::ParallelAdjacentSides),
        ));
    }

    #[test]
    fn polygon_new_fails_when_vertices_are_not_in_the_same_plane() {
        assert!(matches!(
            Polygon::new([
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(2.0), Val(1.0)),
                Point::new(Val(-1.0), Val(1.0), Val(3.0)),
                Point::new(Val(0.0), Val(-1.0), Val(5.0)),
            ]),
            Err(TryNewPolygonError::NotFlat),
        ));
    }

    #[test]
    fn polygon_hit_succeeds() {
        let polygon = Polygon::new([
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(1.0)),
            Point::new(Val(-1.0), Val(1.0), Val(3.0)),
            Point::new(Val(0.0), Val(-1.0), Val(2.0)),
        ])
        .unwrap();

        let ray = Ray::new(
            Point::new(Val(-2.0), Val(0.0), Val(2.0)),
            UnitVector::x_direction(),
        );

        let intersection = polygon.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(intersection.distance(), Val(1.8));
        assert_eq!(
            intersection.position(),
            Point::new(Val(-0.2), Val(0.0), Val(2.0)),
        );
        assert_eq!(
            intersection.normal(),
            Vector::new(
                Val(-0.8451542547285166),
                Val(-0.1690308509457033),
                Val(-0.50709255283711),
            )
            .normalize()
            .unwrap(),
        );
        assert_eq!(intersection.side(), SurfaceSide::Back);
    }

    #[test]
    fn polygon_hit_succeeds_returning_none() {
        let polygon = Polygon::new([
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(1.0)),
            Point::new(Val(-1.0), Val(1.0), Val(3.0)),
            Point::new(Val(0.0), Val(-1.0), Val(2.0)),
        ])
        .unwrap();

        let ray = Ray::new(
            Point::new(Val(0.0), Val(1.0), Val(0.0)),
            UnitVector::x_direction(),
        );
        assert!(polygon.hit(&ray, DisRange::positive()).is_none());
    }

    #[test]
    fn polygon_area_succeeds() {
        let polygon = Polygon::new([
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(-1.0), Val(0.0), Val(0.0)),
            Point::new(Val(-1.0), Val(2.0), Val(0.0)),
            Point::new(Val(0.0), Val(1.0), Val(0.0)),
            Point::new(Val(1.0), Val(2.0), Val(0.0)),
        ])
        .unwrap();
        assert_eq!(polygon.area(), Val(3.0));
    }

    #[test]
    fn polygon_bounding_box_succeeds() {
        let polygon = Polygon::new([
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(1.0)),
            Point::new(Val(-1.0), Val(1.0), Val(3.0)),
            Point::new(Val(0.0), Val(-1.0), Val(2.0)),
        ])
        .unwrap();

        assert_eq!(
            polygon.bounding_box(),
            Some(BoundingBox::new(
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(2.0), Val(3.0)),
            )),
        )
    }

    #[test]
    fn polygon_trangulate_succeeds() {
        let polygon = Polygon::new([
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(1.0)),
            Point::new(Val(-1.0), Val(1.0), Val(3.0)),
            Point::new(Val(0.0), Val(-1.0), Val(2.0)),
        ])
        .unwrap();

        let triangles = polygon.triangulate();
        assert_eq!(triangles.len(), 2);
    }
}
