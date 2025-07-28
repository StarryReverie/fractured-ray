use crate::domain::math::algebra::{UnitVector, Vector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;

use super::Photon;

#[derive(Debug, Clone, PartialEq)]
pub struct PhotonMap {
    nodes: Vec<KdTreeNode>,
}

impl PhotonMap {
    pub fn build(mut photons: Vec<Photon>) -> Self {
        let mut nodes = vec![KdTreeNode::default(); photons.len()];
        Self::build_impl(&mut photons, &mut nodes, 0);
        Self { nodes }
    }

    fn build_impl(photons: &mut [Photon], nodes: &mut [KdTreeNode], index: usize) {
        if index >= nodes.len() {
            return;
        }
        let axis = Self::select_split_axis(photons);
        let (left_size, _right_size) = Self::calc_subtree_size(photons.len());
        let (left, current, right) =
            photons.select_nth_unstable_by_key(left_size, |photon| photon.position().axis(axis));
        nodes[index] = KdTreeNode::new(current.clone(), axis as u8);
        Self::build_impl(left, nodes, index * 2 + 1);
        Self::build_impl(right, nodes, index * 2 + 2);
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

    fn calc_subtree_size(size: usize) -> (usize, usize) {
        let exp2 = (size + 2).next_power_of_two() / 2;
        let right = exp2 / 2 - 1;
        let left = size - 1 - right;
        (left, right)
    }

    pub fn search(&self, center: Point, radius: Val) -> Vec<&Photon> {
        let mut res = Vec::new();
        self.search_impl(0, center, radius.powi(2), &mut res);
        res
    }

    fn search_impl<'a>(
        &'a self,
        index: usize,
        center: Point,
        radius_squared: Val,
        res: &mut Vec<&'a Photon>,
    ) {
        if index >= self.nodes.len() {
            return;
        }

        let photon = &self.nodes[index].photon;
        let dis_squared = (center - photon.position()).norm_squared();
        if dis_squared <= radius_squared {
            res.push(photon);
        }

        let axis = self.nodes[index].axis as usize;
        let axis_dis = center.axis(axis) - photon.position().axis(axis);
        let (near, far) = if axis_dis < Val(0.0) {
            (2 * index + 1, 2 * index + 2)
        } else {
            (2 * index + 2, 2 * index + 1)
        };

        self.search_impl(near, center, radius_squared, res);
        if axis_dis.powi(2) <= radius_squared {
            self.search_impl(far, center, radius_squared, res);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct KdTreeNode {
    photon: Photon,
    axis: u8,
}

impl KdTreeNode {
    fn new(photon: Photon, axis: u8) -> Self {
        Self { photon, axis }
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
