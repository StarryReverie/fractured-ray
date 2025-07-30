use crate::domain::math::algebra::{UnitVector, Vector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;

use super::Photon;

#[derive(Debug, Clone, PartialEq)]
pub struct PhotonMap {
    nodes: Vec<KdTreeNode>,
    root: Option<usize>,
}

impl PhotonMap {
    pub fn build(mut photons: Vec<Photon>) -> Self {
        let mut nodes = vec![KdTreeNode::default(); photons.len()];
        let root = Self::build_impl(&mut photons, &mut nodes, 0);
        Self { nodes, root }
    }

    fn build_impl(
        photons: &mut [Photon],
        nodes: &mut [KdTreeNode],
        offset: usize,
    ) -> Option<usize> {
        if photons.is_empty() {
            return None;
        }

        let mid = photons.len() / 2;
        let axis = Self::select_split_axis(photons);
        photons.select_nth_unstable_by_key(mid, |photon| photon.position().axis(axis));
        let (pl, pm, pr) = Self::split_at_mid(photons, mid);
        let (nl, nm, nr) = Self::split_at_mid(nodes, mid);

        let (left, right) = rayon::join(
            || Self::build_impl(pl, nl, offset),
            || Self::build_impl(pr, nr, offset + mid + 1),
        );
        *nm = KdTreeNode::new(pm.clone(), axis as u8, left, right);
        Some(offset + mid)
    }

    fn select_split_axis(photons: &[Photon]) -> usize {
        let min_init = Point::new(Val::INFINITY, Val::INFINITY, Val::INFINITY);
        let max_init = Point::new(-Val::INFINITY, -Val::INFINITY, -Val::INFINITY);
        let (min, max) = (photons.iter())
            .map(|photon| photon.position())
            .fold((min_init, max_init), |(min, max), position| {
                (min.component_min(&position), max.component_max(&position))
            });

        let diag = max - min;
        let max_component = (diag.x()).max(diag.y()).max(diag.z());
        if max_component == diag.x() {
            0
        } else if max_component == diag.y() {
            1
        } else {
            2
        }
    }

    fn split_at_mid<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut T, &mut [T]) {
        assert!(mid < slice.len());
        let (left, rest) = slice.split_at_mut(mid);
        let (center, right) = rest.split_first_mut().unwrap();
        (left, center, right)
    }

    pub fn search(&self, center: Point, radius: Val) -> Vec<&Photon> {
        let Some(root) = self.root else {
            return Vec::new();
        };

        let radius2 = radius.powi(2);
        let mut res = Vec::new();
        let mut planned = Vec::with_capacity(64);
        planned.push(root);

        while let Some(index) = planned.pop() {
            assert!(index < self.nodes.len());

            let photon = &self.nodes[index].photon;
            if (center - photon.position()).norm_squared() <= radius2 {
                res.push(photon);
            }

            let axis = self.nodes[index].axis as usize;
            let axis_dis = center.axis(axis) - photon.position().axis(axis);
            let (near, far) = match (axis_dis < Val(0.0), axis_dis.powi(2) <= radius2) {
                (true, true) => (self.nodes[index].left(), self.nodes[index].right()),
                (true, false) => (self.nodes[index].left(), None),
                (false, true) => (self.nodes[index].right(), self.nodes[index].left()),
                (false, false) => (self.nodes[index].right(), None),
            };

            match (near, far) {
                (Some(near), None) => planned.push(near),
                (None, Some(far)) => planned.push(far),
                (Some(near), Some(far)) => {
                    planned.push(far);
                    planned.push(near);
                }
                (None, None) => {}
            }
        }

        res
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct KdTreeNode {
    photon: Photon,
    axis: u8,
    left: u32,
    right: u32,
}

impl KdTreeNode {
    const NONE: u32 = u32::MAX;

    fn new(photon: Photon, axis: u8, left: Option<usize>, right: Option<usize>) -> Self {
        Self {
            photon,
            axis,
            left: left.map(|v| v as u32).unwrap_or(Self::NONE),
            right: right.map(|v| v as u32).unwrap_or(Self::NONE),
        }
    }

    fn left(&self) -> Option<usize> {
        if self.left != Self::NONE {
            Some(self.left as usize)
        } else {
            None
        }
    }

    fn right(&self) -> Option<usize> {
        if self.right != Self::NONE {
            Some(self.right as usize)
        } else {
            None
        }
    }
}

impl Default for KdTreeNode {
    fn default() -> Self {
        Self::new(
            Photon::new(
                Point::default(),
                UnitVector::x_direction(),
                Vector::default(),
            ),
            0,
            None,
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photon_map_search_succeeds() {
        let photons = vec![
            create_photon(4.0, 0.0, 0.0),
            create_photon(3.0, 3.0, 1.0),
            create_photon(0.0, 0.0, 0.0),
            create_photon(-2.0, -3.0, -1.0),
            create_photon(3.0, -3.0, 2.0),
        ];

        let photon_map = PhotonMap::build(photons);
        let res = dbg!(photon_map.search(Point::new(Val(2.0), Val(-1.0), Val(0.0)), Val(3.0)));
        assert!(
            res.iter()
                .find(|p| p.position() == Point::new(Val(0.0), Val(0.0), Val(0.0)))
                .is_some()
        );
        assert!(
            res.iter()
                .find(|p| p.position() == Point::new(Val(4.0), Val(0.0), Val(0.0)))
                .is_some()
        );
    }

    fn create_photon(x: f64, y: f64, z: f64) -> Photon {
        Photon::new(
            Point::new(Val(x), Val(y), Val(z)),
            UnitVector::x_direction(),
            Vector::default(),
        )
    }
}
