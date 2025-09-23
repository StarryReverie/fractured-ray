use std::sync::Arc;

use crate::domain::math::geometry::Point;
use crate::domain::math::transformation::Sequential;
use crate::domain::shape::primitive::{MeshPolygon, MeshTriangle};
use crate::domain::shape::util::{ShapeConstructor, ShapeContainer, ShapeId};
use crate::domain::texture::def::UvCoordinate;

use super::{MeshData, MeshDataComponent, TryAddMeshUvCoordinateError, TryNewMeshError};

#[derive(Debug, Clone)]
pub struct MeshConstructor {
    vertices: MeshDataComponent<Point>,
    uvs: Option<MeshDataComponent<UvCoordinate>>,
}

impl MeshConstructor {
    pub fn new<V>(vertices: V, indices: Vec<Vec<usize>>) -> Result<Self, TryNewMeshError>
    where
        V: Into<Arc<[Point]>>,
    {
        Ok(Self {
            vertices: MeshDataComponent::<Point>::new(vertices, indices)?,
            uvs: None,
        })
    }

    pub fn with_uvs<U>(
        self,
        uvs: U,
        indices: Vec<Vec<usize>>,
    ) -> Result<Self, TryAddMeshUvCoordinateError>
    where
        U: Into<Arc<[UvCoordinate]>>,
    {
        Ok(Self {
            uvs: Some(MeshDataComponent::<UvCoordinate>::new(
                uvs,
                indices,
                self.vertices.triangles().len(),
                self.vertices.polygons().len(),
            )?),
            ..self
        })
    }

    pub fn construct_impl(
        self,
        transformation: Option<Sequential>,
    ) -> (Vec<MeshTriangle>, Vec<MeshPolygon>) {
        let data = Arc::new(MeshData::new(
            self.vertices.clone(),
            self.uvs.clone(),
            transformation,
        ));

        let mesh_triangles = (0..data.vertices().triangles().len())
            .map(|index| MeshTriangle::new(data.clone(), index))
            .collect();

        let mesh_polygons = (0..data.vertices().polygons().len())
            .map(|index| MeshPolygon::new(data.clone(), index))
            .collect();

        (mesh_triangles, mesh_polygons)
    }
}

impl ShapeConstructor for MeshConstructor {
    fn construct(self: Box<Self>, container: &mut dyn ShapeContainer) -> Vec<ShapeId> {
        let (triangles, polygons) = self.construct_impl(None);

        let mut ids = Vec::with_capacity(triangles.len() + polygons.len());
        for triangle in triangles {
            ids.push(container.add_shape(triangle.into()));
        }
        for polygon in polygons {
            ids.push(container.add_shape(polygon.into()));
        }
        ids
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::numeric::Val;

    use super::*;

    #[test]
    fn mesh_constructor_new_succeeds() {
        let (triangles, polygons) = MeshConstructor::new(
            vec![
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(2.0)),
            ],
            vec![
                vec![0, 1, 2, 3],
                vec![0, 1, 4],
                vec![1, 2, 4],
                vec![2, 3, 4],
                vec![3, 1, 4],
            ],
        )
        .unwrap()
        .construct_impl(None);

        assert_eq!(triangles.len(), 4);
        assert_eq!(polygons.len(), 1);
    }

    #[test]
    fn mesh_constructor_with_uvs_succeeds() {
        let (triangles, polygons) = MeshConstructor::new(
            vec![
                Point::new(Val(1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(1.0), Val(0.0)),
                Point::new(Val(-1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(1.0), Val(-1.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(2.0)),
            ],
            vec![
                vec![0, 1, 2, 3],
                vec![0, 1, 4],
                vec![1, 2, 4],
                vec![2, 3, 4],
                vec![3, 1, 4],
            ],
        )
        .unwrap()
        .with_uvs(
            vec![
                UvCoordinate::new(Val(1.0), Val(1.0)).unwrap(),
                UvCoordinate::new(Val(0.0), Val(1.0)).unwrap(),
                UvCoordinate::new(Val(0.0), Val(0.0)).unwrap(),
                UvCoordinate::new(Val(1.0), Val(0.0)).unwrap(),
                UvCoordinate::new(Val(0.5), Val(0.5)).unwrap(),
            ],
            vec![
                vec![0, 1, 2, 3],
                vec![0, 1, 4],
                vec![1, 2, 4],
                vec![2, 3, 4],
                vec![3, 1, 4],
            ],
        )
        .unwrap()
        .construct_impl(None);

        assert_eq!(triangles.len(), 4);
        assert_eq!(polygons.len(), 1);
    }
}
