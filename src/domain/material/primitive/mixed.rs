use std::collections::HashMap;

use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::material::def::{DynMaterial, Material, MaterialCategory, MaterialKind};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};

use super::Emissive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mixed {
    emissive: Option<Box<Emissive>>,
    other: Option<Box<OtherMixed>>,
}

impl Mixed {
    pub fn builder() -> MixedBuilder {
        MixedBuilder::new()
    }
}

impl Material for Mixed {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Mixed
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        let mut res = Contribution::new();

        if let Some(emissive) = &self.emissive {
            res = res + emissive.shade(context, state.clone(), ray, intersection);
        }

        match self.other.as_ref().map(AsRef::as_ref) {
            Some(OtherMixed::Singleton { inner }) => {
                res = res + inner.shade(context, state, ray, intersection);
            }
            Some(OtherMixed::Microfacet {
                diffuse,
                microfacet,
            }) => {
                const SELECT_DIFFUSE_PROB: Val = Val(0.5);
                if Val(context.rng().random()) < SELECT_DIFFUSE_PROB {
                    let diffuse_res = diffuse.shade(context, state, ray, intersection);
                    res = res + diffuse_res * SELECT_DIFFUSE_PROB.recip();
                } else {
                    let microfacet_res = microfacet.shade(context, state, ray, intersection);
                    res = res + microfacet_res * (Val(1.0) - SELECT_DIFFUSE_PROB).recip();
                }
            }
            None => {}
        }

        res
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: &PhotonRay,
        intersection: &RayIntersection,
    ) {
        match self.other.as_ref().map(AsRef::as_ref) {
            Some(OtherMixed::Singleton { inner }) => {
                inner.receive(context, state, photon, intersection);
            }
            Some(OtherMixed::Microfacet {
                diffuse,
                microfacet,
            }) => {
                const SELECT_DIFFUSE_PROB: Val = Val(0.5);
                let photon = photon.clone();
                if Val(context.rng().random()) < SELECT_DIFFUSE_PROB {
                    let photon = photon.scale_throughput(SELECT_DIFFUSE_PROB.recip());
                    diffuse.receive(context, state, &photon, intersection);
                } else {
                    let photon = photon.scale_throughput((Val(1.0) - SELECT_DIFFUSE_PROB).recip());
                    microfacet.receive(context, state, &photon, intersection);
                }
            }
            None => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixedBuilder {
    materials: Vec<DynMaterial>,
}

impl MixedBuilder {
    pub fn new() -> Self {
        Self {
            materials: Vec::new(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add<M>(mut self, material: M) -> Self
    where
        M: Into<DynMaterial>,
    {
        self.materials.push(material.into());
        self
    }

    pub fn build(self) -> Result<Mixed, TryBuildMixedError> {
        let mut by_category = HashMap::with_capacity(self.materials.len());

        for material in self.materials {
            let category = material.kind().category();
            ensure!(category != MaterialCategory::Mixed, NestedMixedSnafu);

            let prev = by_category.insert(category, material);
            ensure!(prev.is_none(), DuplicatedCategorySnafu { category });
        }

        let emissive = by_category
            .remove(&MaterialCategory::Emissive)
            .map(|material| {
                let DynMaterial::Emissive(emissive) = material else {
                    unreachable!()
                };
                Box::new(emissive)
            });

        let other = if by_category.is_empty() {
            None
        } else if by_category.len() == 1 {
            let inner = by_category.into_values().next().unwrap();
            Some(Box::new(OtherMixed::Singleton { inner }))
        } else if by_category.len() == 2 {
            let diffuse = by_category.get(&MaterialCategory::Diffuse);
            let microfacet = by_category.get(&MaterialCategory::Microfacet);

            if let Some((diffuse, microfacet)) = diffuse.zip(microfacet) {
                Some(Box::new(OtherMixed::Microfacet {
                    diffuse: diffuse.clone(),
                    microfacet: microfacet.clone(),
                }))
            } else {
                return InvalidCombinationSnafu {
                    categories: by_category.into_keys().collect::<Vec<_>>(),
                }
                .fail();
            }
        } else {
            return InvalidCombinationSnafu {
                categories: by_category.into_keys().collect::<Vec<_>>(),
            }
            .fail();
        };

        Ok(Mixed { emissive, other })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OtherMixed {
    Singleton {
        inner: DynMaterial,
    },
    Microfacet {
        diffuse: DynMaterial,
        microfacet: DynMaterial,
    },
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryBuildMixedError {
    #[snafu(display("could not mix more than one materials of the same category"))]
    DuplicatedCategory { category: MaterialCategory },
    #[snafu(display("could not add another nested mixed material to the outer one"))]
    NestedMixed,
    #[snafu(display("could not combine {categories:?} to a mixed material"))]
    InvalidCombination { categories: Vec<MaterialCategory> },
}
