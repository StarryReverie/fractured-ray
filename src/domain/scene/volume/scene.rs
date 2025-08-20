use std::collections::HashMap;
use std::ops::{Bound, RangeBounds};

use rand::prelude::*;

use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::medium::def::medium::MediumId;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, SurfaceSide};
use crate::domain::scene::bvh::Bvh;
use crate::domain::scene::pool::BoundaryPool;
use crate::domain::shape::def::ShapeContainer;

use super::{BoundaryId, MediumSegment, VolumeScene};

#[derive(Debug)]
pub struct BvhVolumeScene {
    boundaries: Box<BoundaryPool>,
    bvh: Bvh<BoundaryId>,
    outer_media: HashMap<MediumId, Option<MediumId>>,
}

impl BvhVolumeScene {
    const OUTER_MEDIUM_MAX_DETECTION_COUNT: usize = 16;

    fn new(boundaries: Box<BoundaryPool>, ids: &[BoundaryId]) -> Self {
        let mut bboxes = Vec::with_capacity(ids.len());

        for id in ids {
            let sid = id.shape_id();
            if let Some(bbox) = boundaries.get_shape(sid).unwrap().bounding_box() {
                bboxes.push((*id, bbox));
            }
        }

        let bvh = Bvh::new(bboxes, Vec::new());
        let outer_media = Self::determine_outer_media(&boundaries, ids, &bvh);

        Self {
            boundaries,
            bvh,
            outer_media,
        }
    }

    fn determine_outer_media(
        boundaries: &BoundaryPool,
        ids: &[BoundaryId],
        bvh: &Bvh<BoundaryId>,
    ) -> HashMap<MediumId, Option<MediumId>> {
        let mut rng = rand::rng();
        let mut boundary_ids_map: HashMap<MediumId, Vec<BoundaryId>> = HashMap::new();
        for id in ids {
            let bids = boundary_ids_map.entry(id.medium_id()).or_default();
            bids.push(*id);
        }

        let mut outer_medium = HashMap::new();
        for (medium_id, boundary_ids) in boundary_ids_map.iter() {
            let mut outer = None;

            for _ in 0..Self::OUTER_MEDIUM_MAX_DETECTION_COUNT {
                let bid = boundary_ids[rng.random_range(0..boundary_ids.len())];
                let boundary = boundaries.get_shape(bid.shape_id()).unwrap();
                let Some(sampler) = boundary.get_point_sampler(bid.shape_id()) else {
                    break;
                };

                let start_sample = sampler.sample_point(&mut rng).unwrap();
                let direction = loop {
                    let direction = UnitVector::random(&mut rng);
                    if direction.dot(start_sample.normal()) > Val(0.0) {
                        break direction;
                    }
                };
                let detection_ray = Ray::new(start_sample.point(), direction);

                let res = bvh.search(&detection_ray, DisRange::positive(), boundaries);
                if let Some((isect, isect_id)) = res {
                    if isect.side() == SurfaceSide::Back {
                        outer = Some(isect_id.medium_id());
                        break;
                    }
                } else {
                    break;
                }
            }

            outer_medium.insert(*medium_id, outer);
        }
        outer_medium
    }

    fn determine_initial_medium(
        &self,
        ray: &Ray,
        isects: &[(RayIntersection, BoundaryId)],
    ) -> Option<MediumId> {
        let (side, id) = if let Some((first_isect, id)) = isects.first() {
            (first_isect.side(), *id)
        } else {
            let range = DisRange::positive();
            let (isect, id) = self.bvh.search(ray, range, &*self.boundaries)?;
            (isect.side(), id)
        };
        if side == SurfaceSide::Front {
            self.outer_media.get(&id.medium_id()).cloned().unwrap()
        } else {
            Some(id.medium_id())
        }
    }
}

impl VolumeScene for BvhVolumeScene {
    fn find_segments(&self, ray: &Ray, range: DisRange) -> Vec<MediumSegment> {
        let mut isects = self.bvh.search_all(ray, range, &*self.boundaries);
        isects.sort_by_key(|i| i.0.distance());

        let mut res = Vec::with_capacity(isects.len());
        let mut current_medium = self.determine_initial_medium(ray, &isects);
        let mut last_distance = match range.start_bound() {
            Bound::Included(v) | Bound::Excluded(v) => *v,
            Bound::Unbounded => unreachable!("range's start bound should not be unbounded"),
        };
        let mut last_endpoint = ray.at(last_distance);
        println!("{}", isects.len());

        for (isect, id) in &isects {
            if let Some(current_medium) = current_medium {
                let length = isect.distance() - last_distance;
                if length > Val(0.0) {
                    res.push(MediumSegment::new(last_endpoint, length, current_medium));
                }
            }

            current_medium = if isect.side() == SurfaceSide::Front {
                Some(id.medium_id())
            } else {
                self.outer_media.get(&id.medium_id()).cloned().unwrap()
            };
            last_endpoint = isect.position();
            last_distance = isect.distance();
        }

        if let Some(current_medium) = current_medium {
            let max_distance = match range.end_bound() {
                Bound::Included(v) | Bound::Excluded(v) => *v,
                Bound::Unbounded => Val::INFINITY,
            };
            let length = max_distance - last_distance;
            if length > Val(0.0) {
                res.push(MediumSegment::new(last_endpoint, length, current_medium));
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::color::{Albedo, Spectrum};
    use crate::domain::math::geometry::Point;
    use crate::domain::medium::def::medium::MediumContainer;
    use crate::domain::medium::primitive::Isotropic;
    use crate::domain::shape::primitive::Aabb;

    use super::*;

    #[test]
    fn bvh_volume_scene_find_segments_succeeds_when_ray_starts_outside() {
        let (scene, ids) = get_test_bvh_volume_scene();

        let ray = Ray::new(
            Point::new(Val(-0.5), Val(0.5), Val(0.5)),
            UnitVector::x_direction(),
        );

        let segments = dbg!(scene.find_segments(&ray, DisRange::positive()));
        let mut iter = segments.iter();

        let segment = iter.next().unwrap();
        assert_eq!(segment.start(), Point::new(Val(0.0), Val(0.5), Val(0.5)));
        assert_eq!(segment.length(), Val(1.0));
        assert_eq!(segment.medium(), ids[0].medium_id());

        let segment = iter.next().unwrap();
        assert_eq!(segment.length(), Val(3.0));
        assert_eq!(segment.medium(), ids[1].medium_id());

        let segment = iter.next().unwrap();
        assert_eq!(segment.length(), Val(1.0));
        assert_eq!(segment.medium(), ids[0].medium_id());

        let segment = iter.next().unwrap();
        assert_eq!(segment.length(), Val(4.0));
        assert_eq!(segment.medium(), ids[2].medium_id());

        let segment = iter.next().unwrap();
        assert_eq!(segment.length(), Val(1.0));
        assert_eq!(segment.medium(), ids[0].medium_id());
    }

    #[test]
    fn bvh_volume_scene_find_segments_succeeds_when_ray_starts_inside() {
        let (scene, ids) = get_test_bvh_volume_scene();

        let ray = Ray::new(
            Point::new(Val(0.1), Val(0.5), Val(0.5)),
            UnitVector::x_direction(),
        );

        let segments = dbg!(scene.find_segments(&ray, DisRange::positive()));
        let mut iter = segments.iter();

        let segment = iter.next().unwrap();
        assert_eq!(segment.start(), Point::new(Val(0.1), Val(0.5), Val(0.5)));
        assert_eq!(segment.length(), Val(0.9));
        assert_eq!(segment.medium(), ids[0].medium_id());

        let segment = iter.next().unwrap();
        assert_eq!(segment.length(), Val(3.0));
        assert_eq!(segment.medium(), ids[1].medium_id());

        let segment = iter.next().unwrap();
        assert_eq!(segment.length(), Val(1.0));
        assert_eq!(segment.medium(), ids[0].medium_id());

        let segment = iter.next().unwrap();
        assert_eq!(segment.length(), Val(4.0));
        assert_eq!(segment.medium(), ids[2].medium_id());

        let segment = iter.next().unwrap();
        assert_eq!(segment.length(), Val(1.0));
        assert_eq!(segment.medium(), ids[0].medium_id());
    }

    fn get_test_bvh_volume_scene() -> (BvhVolumeScene, Vec<BoundaryId>) {
        let mut boundaries = Box::new(BoundaryPool::new());
        let mut ids = Vec::new();

        let shape_id = boundaries.add_shape(Aabb::new(
            Point::new(Val(0.0), Val(-1.0), Val(-1.0)),
            Point::new(Val(10.0), Val(2.0), Val(2.0)),
        ));
        let medium_id = boundaries
            .add_medium(Isotropic::new(Albedo::WHITE, Spectrum::broadcast(Val(1.0))).unwrap());
        ids.push(BoundaryId::new(shape_id, medium_id));

        let shape_id = boundaries.add_shape(Aabb::new(
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(4.0), Val(1.0), Val(1.0)),
        ));
        let medium_id = boundaries
            .add_medium(Isotropic::new(Albedo::WHITE, Spectrum::broadcast(Val(1.0))).unwrap());
        ids.push(BoundaryId::new(shape_id, medium_id));

        let shape_id = boundaries.add_shape(Aabb::new(
            Point::new(Val(5.0), Val(0.0), Val(0.0)),
            Point::new(Val(9.0), Val(1.0), Val(1.0)),
        ));
        let medium_id = boundaries
            .add_medium(Isotropic::new(Albedo::WHITE, Spectrum::broadcast(Val(1.0))).unwrap());
        ids.push(BoundaryId::new(shape_id, medium_id));

        let scene = BvhVolumeScene::new(boundaries, &ids);
        (scene, ids)
    }
}
