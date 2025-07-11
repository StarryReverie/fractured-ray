use smallvec::SmallVec;

use crate::domain::geometry::{Point, Val};
use crate::domain::ray::Ray;

use super::material::Material;
use super::shape::{
    BoundingBox, DisRange, MeshConstructor, RayIntersection, Shape, TryNewMeshError,
};
use super::{EntityId, EntityPool};

#[derive(Debug)]
pub struct SceneBuilder {
    entities: Box<EntityPool>,
    ids: Vec<EntityId>,
}

impl SceneBuilder {
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

    pub fn add_mesh<M: Material>(
        &mut self,
        vertices: Vec<Point>,
        vertex_indices: Vec<Vec<usize>>,
        material: M,
    ) -> Result<&mut Self, TryNewMeshError> {
        let ctor = MeshConstructor::new(vertices, vertex_indices)?;
        let (triangles, polygons) = ctor.construct();

        let material_id = self.entities.add_material(material);

        for triangle in triangles {
            let shape_id = self.entities.add_shape(triangle);
            self.ids.push(EntityId::new(shape_id, material_id));
        }

        for polygon in polygons {
            let shape_id = self.entities.add_shape(polygon);
            self.ids.push(EntityId::new(shape_id, material_id));
        }

        Ok(self)
    }

    pub fn build(self) -> Scene {
        let mut nodes = Vec::with_capacity(self.ids.len() * 2);
        let mut bboxes = Vec::with_capacity(self.ids.len());
        let mut unboundeds = Vec::new();

        for id in self.ids {
            match self.entities.get_shape(id).unwrap().bounding_box() {
                Some(bbox) => bboxes.push((id, bbox)),
                None => unboundeds.push(id),
            }
        }

        if !bboxes.is_empty() {
            Self::build_bvh(&mut nodes, bboxes);
        }

        Scene {
            entities: self.entities,
            nodes,
            unboundeds,
        }
    }

    fn build_bvh(nodes: &mut Vec<BvhNode>, bboxes: Vec<(EntityId, BoundingBox)>) -> usize {
        if bboxes.len() == 1 {
            let (id, bbox) = bboxes
                .into_iter()
                .next()
                .expect("bboxes should have at least one element");
            nodes.push(BvhNode::leaf(bbox, id));
            return nodes.len() - 1;
        }

        let bbox_num = bboxes.len();
        let node_bbox = Self::merge_bboxes(bboxes.iter().map(|bbox| &bbox.1))
            .expect("bboxes should have at least one element");
        let axis = Self::select_bbox_partition_axis(&node_bbox);
        let mut partition = Self::partition_bboxes(axis, &node_bbox, bboxes);

        if let Some(mid) = Self::calc_split_point(&partition, bbox_num, node_bbox.surface_area()) {
            nodes.push(BvhNode::internal(node_bbox));
            let node_id = nodes.len() - 1;

            let right_bboxes = partition.drain(mid..).flat_map(|t| t.items).collect();
            let left_bboxes = partition.into_iter().flat_map(|t| t.items).collect();
            let _left = Self::build_bvh(nodes, left_bboxes);
            let right = Self::build_bvh(nodes, right_bboxes);

            let BvhNode::Internal { right: r, .. } = &mut nodes[node_id] else {
                unreachable!("nodes[node_id] was constructed as BvhNode::Internal")
            };
            *r = right;

            node_id
        } else {
            let ids = partition.into_iter().flat_map(|t| t.items).map(|t| t.0);
            nodes.push(BvhNode::cluster_leaf(node_bbox, ids));
            nodes.len() - 1
        }
    }

    fn select_bbox_partition_axis(bbox: &BoundingBox) -> usize {
        let diag = bbox.max() - bbox.min();
        let max_component = (diag.x()).max(diag.y()).max(diag.z());
        if max_component == diag.x() {
            0
        } else if max_component == diag.y() {
            1
        } else {
            2
        }
    }

    fn partition_bboxes(
        axis: usize,
        node_bbox: &BoundingBox,
        bboxes: Vec<(EntityId, BoundingBox)>,
    ) -> Vec<PartitionBucket> {
        let mut buckets = Vec::new();
        buckets.resize(BvhNode::SAH_PARTITION, PartitionBucket::new());
        let range = (node_bbox.min().axis(axis), node_bbox.max().axis(axis));
        let bucket_span = (range.1 - range.0) / BvhNode::SAH_PARTITION.into();

        for (id, bbox) in bboxes {
            let fraction = (bbox.centroid().axis(axis) - range.0) / bucket_span;
            let index = usize::from(fraction).clamp(0, BvhNode::SAH_PARTITION - 1);
            buckets[index].items.push((id, bbox));
        }

        for bucket in &mut buckets {
            bucket.overall_bbox = Self::merge_bboxes(bucket.items.iter().map(|bbox| &bbox.1));
        }

        buckets
    }

    fn calc_split_point(
        partition: &[PartitionBucket],
        bbox_num: usize,
        total_surface_area: Val,
    ) -> Option<usize> {
        assert_eq!(partition.len(), BvhNode::SAH_PARTITION);
        let mut cost = [BvhNode::TRAVERSAL_COST; BvhNode::SAH_PARTITION - 1];

        let mut merged_bbox: Option<BoundingBox> = None;
        let mut num = 0;
        let mut num_pre = [0; BvhNode::SAH_PARTITION - 1];
        for i in 0..BvhNode::SAH_PARTITION - 1 {
            num += partition[i].items.len();
            num_pre[i] = num;
            merged_bbox = merged_bbox
                .map(|bbox| partition[i].merge_bbox(bbox))
                .or_else(|| partition[i].overall_bbox.clone());
            let surface_area = merged_bbox
                .as_ref()
                .map_or(Val(0.0), BoundingBox::surface_area);
            cost[i] +=
                BvhNode::INTERSECTION_COST * Val::from(num) * surface_area / total_surface_area;
        }

        num = 0;
        merged_bbox = None;
        let mut num_suf = [0; BvhNode::SAH_PARTITION - 1];
        for i in (0..BvhNode::SAH_PARTITION - 1).rev() {
            num += partition[i + 1].items.len();
            num_suf[i] = num;
            merged_bbox = merged_bbox
                .map(|bbox| partition[i].merge_bbox(bbox))
                .or_else(|| partition[i].overall_bbox.clone());
            let surface_area = merged_bbox
                .as_ref()
                .map_or(Val(0.0), BoundingBox::surface_area);
            cost[i] +=
                BvhNode::INTERSECTION_COST * Val::from(num) * surface_area / total_surface_area;
        }

        let mut res = 0;
        for i in 1..BvhNode::SAH_PARTITION - 1 {
            if cost[i] < cost[res] {
                res = i
            } else if cost[i] == cost[res] {
                let i_diff = num_pre[i].abs_diff(num_suf[i]);
                let optimal_diff = num_pre[res].abs_diff(num_suf[res]);

                if i_diff < optimal_diff {
                    res = i
                }
            }
        }

        let leaf_cost = Val::from(bbox_num) * BvhNode::TRAVERSAL_COST;
        if num_pre[res] != 0 && num_suf[res] != 0 && cost[res] < leaf_cost {
            Some(res + 1)
        } else {
            None
        }
    }

    fn merge_bboxes<'a, I>(mut bboxes: I) -> Option<BoundingBox>
    where
        I: Iterator<Item = &'a BoundingBox>,
    {
        let init = bboxes.next().cloned()?;
        let bbox = bboxes.fold(init, |acc, bbox| acc.merge(bbox));
        Some(bbox)
    }
}

#[derive(Clone)]
struct PartitionBucket {
    overall_bbox: Option<BoundingBox>,
    items: Vec<(EntityId, BoundingBox)>,
}

impl PartitionBucket {
    fn new() -> Self {
        Self {
            overall_bbox: None,
            items: Vec::new(),
        }
    }

    fn merge_bbox(&self, other: BoundingBox) -> BoundingBox {
        self.overall_bbox
            .as_ref()
            .map(|s| other.merge(s))
            .unwrap_or(other)
    }
}

#[derive(Debug)]
pub struct Scene {
    entities: Box<EntityPool>,
    nodes: Vec<BvhNode>,
    unboundeds: Vec<EntityId>,
}

impl Scene {
    pub fn find_intersection(
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

    fn find_intersection_with_unboundeds(
        &self,
        ray: &Ray,
        range: DisRange,
    ) -> Option<(RayIntersection, &dyn Material)> {
        self.intersect_for_each(ray, range, self.unboundeds.iter())
    }

    fn find_intersection_in_bvh(
        &self,
        ray: &Ray,
        range: DisRange,
    ) -> Option<(RayIntersection, &dyn Material)> {
        if !self.nodes.is_empty() {
            if self.nodes[0].bounding_box().hit(ray, range).is_some() {
                return self.bvh_search(0, ray, range);
            }
        }
        None
    }

    fn bvh_search(
        &self,
        current: usize,
        ray: &Ray,
        range: DisRange,
    ) -> Option<(RayIntersection, &dyn Material)> {
        assert!(current < self.nodes.len());

        match &self.nodes[current] {
            BvhNode::Internal { right, .. } => {
                let (left, right) = (current + 1, *right);
                let hit_left = self.nodes[left].bounding_box().hit(ray, range);
                let hit_right = self.nodes[right].bounding_box().hit(ray, range);

                match (hit_left, hit_right) {
                    (Some(_), None) => self.bvh_search(left, ray, range),
                    (None, Some(_)) => self.bvh_search(right, ray, range),
                    (Some(dis1), Some(dis2)) => {
                        if dis1 <= dis2 {
                            self.bvh_search(left, ray, range)
                                .map(|res| {
                                    let range = range.shrink_end(res.0.distance());
                                    self.bvh_search(right, ray, range).unwrap_or(res)
                                })
                                .or_else(|| self.bvh_search(right, ray, range))
                        } else {
                            self.bvh_search(right, ray, range)
                                .map(|res| {
                                    let range = range.shrink_end(res.0.distance());
                                    self.bvh_search(left, ray, range).unwrap_or(res)
                                })
                                .or_else(|| self.bvh_search(left, ray, range))
                        }
                    }
                    (None, None) => None,
                }
            }
            BvhNode::Leaf { id, .. } => {
                let shape = self.entities.get_shape(*id).unwrap();
                if let Some(intersection) = shape.hit(ray, range) {
                    let material = self.entities.get_material(*id).unwrap();
                    // println!("id = {id:?}, shape = {shape:#?}, intersection = {intersection:#?}");
                    Some((intersection, material))
                } else {
                    None
                }
            }
            BvhNode::ClusterLeaf { ids, .. } => {
                let ids = ids.iter();
                self.intersect_for_each(ray, range, ids)
            }
        }
    }

    fn intersect_for_each<'a>(
        &self,
        ray: &Ray,
        mut range: DisRange,
        ids: impl Iterator<Item = &'a EntityId>,
    ) -> Option<(RayIntersection, &dyn Material)> {
        let mut closet: Option<(RayIntersection, EntityId)> = None;

        for id in ids {
            let shape = self.entities.get_shape(*id).unwrap();
            if let Some((closet, _)) = &closet {
                range = range.shrink_end(closet.distance());
            }
            if let Some(intersection) = shape.hit(ray, range) {
                closet = Some((intersection, *id));
            };
        }

        if let Some((intersection, id)) = closet {
            let material = self.entities.get_material(id).unwrap();
            Some((intersection, material))
        } else {
            None
        }
    }
}

#[derive(Debug)]
enum BvhNode {
    Internal {
        bounding_box: BoundingBox,
        right: usize,
    },
    Leaf {
        bounding_box: BoundingBox,
        id: EntityId,
    },
    ClusterLeaf {
        bounding_box: BoundingBox,
        ids: Box<SmallVec<[EntityId; 8]>>,
    },
}

impl BvhNode {
    const INDEX_PLACEHOLDER: usize = usize::MAX;
    const SAH_PARTITION: usize = 12;
    const TRAVERSAL_COST: Val = Val(1.0);
    const INTERSECTION_COST: Val = Val(8.0);

    pub fn internal(bounding_box: BoundingBox) -> Self {
        Self::Internal {
            bounding_box,
            right: Self::INDEX_PLACEHOLDER,
        }
    }

    pub fn leaf(bounding_box: BoundingBox, id: EntityId) -> Self {
        Self::Leaf { bounding_box, id }
    }

    pub fn cluster_leaf<I>(bounding_box: BoundingBox, ids: I) -> Self
    where
        I: Iterator<Item = EntityId>,
    {
        Self::ClusterLeaf {
            bounding_box,
            ids: Box::new(ids.collect()),
        }
    }

    pub fn bounding_box(&self) -> &BoundingBox {
        match self {
            BvhNode::Internal { bounding_box, .. } => bounding_box,
            BvhNode::Leaf { bounding_box, .. } => bounding_box,
            BvhNode::ClusterLeaf { bounding_box, .. } => bounding_box,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::color::Color;
    use crate::domain::entity::material::Diffuse;
    use crate::domain::entity::shape::{Polygon, Sphere, Triangle};
    use crate::domain::geometry::Vector;

    use super::*;

    #[test]
    fn scene_build_bvh_succeeds() {
        let mut builder = SceneBuilder::new();
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
        println!("{:#?}", scene.nodes);

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
