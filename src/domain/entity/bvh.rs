use smallvec::SmallVec;

use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{BoundingBox, ShapeContainer, ShapeId};

#[derive(Debug)]
pub struct Bvh<SI>
where
    SI: Eq + Copy + Into<ShapeId>,
{
    nodes: Vec<BvhNode<SI>>,
}

impl<SI> Bvh<SI>
where
    SI: Eq + Copy + Into<ShapeId>,
{
    pub fn new(bboxes: Vec<(SI, BoundingBox)>) -> Self {
        let mut nodes = Vec::with_capacity(bboxes.len() * 2);

        if !bboxes.is_empty() {
            Self::build(&mut nodes, bboxes);
        }

        Self { nodes }
    }

    fn build(nodes: &mut Vec<BvhNode<SI>>, bboxes: Vec<(SI, BoundingBox)>) -> usize {
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
            let _left = Self::build(nodes, left_bboxes);
            let right = Self::build(nodes, right_bboxes);

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
        bboxes: Vec<(SI, BoundingBox)>,
    ) -> Vec<PartitionBucket<SI>> {
        let mut buckets = Vec::new();
        buckets.resize(BvhNode::<SI>::SAH_PARTITION, PartitionBucket::new());
        let range = (node_bbox.min().axis(axis), node_bbox.max().axis(axis));
        let bucket_span = (range.1 - range.0) / BvhNode::<SI>::SAH_PARTITION.into();

        for (id, bbox) in bboxes {
            let fraction = (bbox.centroid().axis(axis) - range.0) / bucket_span;
            let index = usize::from(fraction).clamp(0, BvhNode::<SI>::SAH_PARTITION - 1);
            buckets[index].items.push((id, bbox));
        }

        for bucket in &mut buckets {
            bucket.overall_bbox = Self::merge_bboxes(bucket.items.iter().map(|bbox| &bbox.1));
        }

        buckets
    }

    fn calc_split_point(
        partition: &[PartitionBucket<SI>],
        bbox_num: usize,
        total_surface_area: Val,
    ) -> Option<usize> {
        assert_eq!(partition.len(), BvhNode::<SI>::SAH_PARTITION);
        let mut cost = [BvhNode::<SI>::TRAVERSAL_COST; BvhNode::<ShapeId>::SAH_PARTITION - 1];

        let mut merged_bbox: Option<BoundingBox> = None;
        let mut num = 0;
        let mut num_pre = [0; BvhNode::<ShapeId>::SAH_PARTITION - 1];
        for i in 0..BvhNode::<SI>::SAH_PARTITION - 1 {
            num += partition[i].items.len();
            num_pre[i] = num;
            merged_bbox = merged_bbox
                .map(|bbox| partition[i].merge_bbox(bbox))
                .or_else(|| partition[i].overall_bbox.clone());
            let surface_area = merged_bbox
                .as_ref()
                .map_or(Val(0.0), BoundingBox::surface_area);
            cost[i] += BvhNode::<ShapeId>::INTERSECTION_COST * Val::from(num) * surface_area
                / total_surface_area;
        }

        num = 0;
        merged_bbox = None;
        let mut num_suf = [0; BvhNode::<ShapeId>::SAH_PARTITION - 1];
        for i in (0..BvhNode::<SI>::SAH_PARTITION - 1).rev() {
            num += partition[i + 1].items.len();
            num_suf[i] = num;
            merged_bbox = merged_bbox
                .map(|bbox| partition[i].merge_bbox(bbox))
                .or_else(|| partition[i].overall_bbox.clone());
            let surface_area = merged_bbox
                .as_ref()
                .map_or(Val(0.0), BoundingBox::surface_area);
            cost[i] += BvhNode::<SI>::INTERSECTION_COST * Val::from(num) * surface_area
                / total_surface_area;
        }

        let mut res = 0;
        for i in 1..BvhNode::<SI>::SAH_PARTITION - 1 {
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

        let leaf_cost = Val::from(bbox_num) * BvhNode::<SI>::TRAVERSAL_COST;
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

    pub fn search<SC>(
        &self,
        ray: &Ray,
        range: DisRange,
        shapes: &SC,
    ) -> Option<(RayIntersection, SI)>
    where
        SC: ShapeContainer,
    {
        if !self.nodes.is_empty() {
            if self.nodes[0].bounding_box().try_hit(ray, range).is_some() {
                return self.search_impl(0, ray, range, shapes);
            }
        }
        None
    }

    fn search_impl<SC>(
        &self,
        current: usize,
        ray: &Ray,
        range: DisRange,
        shapes: &SC,
    ) -> Option<(RayIntersection, SI)>
    where
        SC: ShapeContainer,
    {
        assert!(current < self.nodes.len());

        match &self.nodes[current] {
            BvhNode::Internal { right, .. } => {
                let (left, right) = (current + 1, *right);
                let hit_left = self.nodes[left].bounding_box().try_hit(ray, range);
                let hit_right = self.nodes[right].bounding_box().try_hit(ray, range);

                match (hit_left, hit_right) {
                    (Some(_), None) => self.search_impl(left, ray, range, shapes),
                    (None, Some(_)) => self.search_impl(right, ray, range, shapes),
                    (Some(dis1), Some(dis2)) => {
                        if dis1 <= dis2 {
                            self.search_impl(left, ray, range, shapes)
                                .map(|res| {
                                    let range = range.shrink_end(res.0.distance());
                                    self.search_impl(right, ray, range, shapes).unwrap_or(res)
                                })
                                .or_else(|| self.search_impl(right, ray, range, shapes))
                        } else {
                            self.search_impl(right, ray, range, shapes)
                                .map(|res| {
                                    let range = range.shrink_end(res.0.distance());
                                    self.search_impl(left, ray, range, shapes).unwrap_or(res)
                                })
                                .or_else(|| self.search_impl(left, ray, range, shapes))
                        }
                    }
                    (None, None) => None,
                }
            }
            BvhNode::Leaf { id, .. } => {
                let shape = shapes.get_shape((*id).into()).unwrap();
                shape.hit(ray, range).map(|res| (res, *id))
            }
            BvhNode::ClusterLeaf { ids, .. } => {
                let ids = ids.iter();
                self.intersect_for_each(ray, range, ids, shapes)
            }
        }
    }

    fn intersect_for_each<'a, SC, I>(
        &self,
        ray: &Ray,
        mut range: DisRange,
        ids: I,
        shapes: &SC,
    ) -> Option<(RayIntersection, SI)>
    where
        I: Iterator<Item = &'a SI>,
        SC: ShapeContainer,
        SI: 'a,
    {
        let mut closet: Option<(RayIntersection, SI)> = None;
        for id in ids {
            let shape = shapes.get_shape((*id).into()).unwrap();
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

#[derive(Clone)]
struct PartitionBucket<SI>
where
    SI: Eq + Copy + Into<ShapeId>,
{
    overall_bbox: Option<BoundingBox>,
    items: Vec<(SI, BoundingBox)>,
}

impl<SI> PartitionBucket<SI>
where
    SI: Eq + Copy + Into<ShapeId>,
{
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
enum BvhNode<SI>
where
    SI: Eq + Copy + Into<ShapeId>,
{
    Internal {
        bounding_box: BoundingBox,
        right: usize,
    },
    Leaf {
        bounding_box: BoundingBox,
        id: SI,
    },
    ClusterLeaf {
        bounding_box: BoundingBox,
        ids: Box<SmallVec<[SI; 8]>>,
    },
}

impl<SI> BvhNode<SI>
where
    SI: Eq + Copy + Into<ShapeId>,
{
    const INDEX_PLACEHOLDER: usize = usize::MAX;
    const SAH_PARTITION: usize = 12;
    const TRAVERSAL_COST: Val = Val(1.0);
    const INTERSECTION_COST: Val = Val(8.0);

    fn internal(bounding_box: BoundingBox) -> Self {
        Self::Internal {
            bounding_box,
            right: Self::INDEX_PLACEHOLDER,
        }
    }

    fn leaf(bounding_box: BoundingBox, id: SI) -> Self {
        Self::Leaf { bounding_box, id }
    }

    fn cluster_leaf<I>(bounding_box: BoundingBox, ids: I) -> Self
    where
        I: Iterator<Item = SI>,
    {
        Self::ClusterLeaf {
            bounding_box,
            ids: Box::new(ids.collect()),
        }
    }

    fn bounding_box(&self) -> &BoundingBox {
        match self {
            BvhNode::Internal { bounding_box, .. } => bounding_box,
            BvhNode::Leaf { bounding_box, .. } => bounding_box,
            BvhNode::ClusterLeaf { bounding_box, .. } => bounding_box,
        }
    }
}
