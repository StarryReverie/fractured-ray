use getset::{CopyGetters, Getters};
use rand::prelude::*;

use crate::domain::ray::photon::{Photon, PhotonMap, SearchPolicy};
use crate::domain::scene::entity::EntityScene;
use crate::domain::scene::volume::VolumeScene;

use super::{Configuration, Renderer};

#[derive(Getters, CopyGetters)]
pub struct RtContext<'a> {
    #[getset(get_copy = "pub")]
    renderer: &'a dyn Renderer,
    #[getset(get_copy = "pub")]
    entity_scene: &'a dyn EntityScene,
    #[getset(get_copy = "pub")]
    volume_scene: &'a dyn VolumeScene,
    rng: &'a mut dyn RngCore,
    #[getset(get_copy = "pub")]
    config: &'a Configuration,
    #[getset(get = "pub")]
    photon_global: PhotonInfo<'a>,
    #[getset(get = "pub")]
    photon_casutic: PhotonInfo<'a>,
}

impl<'a> RtContext<'a> {
    pub fn new(
        renderer: &'a dyn Renderer,
        entity_scene: &'a dyn EntityScene,
        volume_scene: &'a dyn VolumeScene,
        rng: &'a mut dyn RngCore,
        config: &'a Configuration,
        photon_global: PhotonInfo<'a>,
        photon_casutic: PhotonInfo<'a>,
    ) -> Self {
        Self {
            renderer,
            entity_scene,
            volume_scene,
            rng,
            config,
            photon_global,
            photon_casutic,
        }
    }

    pub fn rng(&mut self) -> &mut &'a mut dyn RngCore {
        &mut self.rng
    }
}

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PhotonInfo<'a> {
    photons: &'a PhotonMap,
    policy: SearchPolicy,
    emitted: usize,
}

impl<'a> PhotonInfo<'a> {
    pub fn new(photons: &'a PhotonMap, policy: SearchPolicy, emitted: usize) -> Self {
        Self {
            photons,
            policy,
            emitted,
        }
    }
}

#[derive(CopyGetters)]
pub struct PmContext<'a> {
    #[getset(get_copy = "pub")]
    renderer: &'a dyn Renderer,
    #[getset(get_copy = "pub")]
    scene: &'a dyn EntityScene,
    rng: &'a mut dyn RngCore,
    photons: &'a mut Vec<Photon>,
}

impl<'a> PmContext<'a> {
    pub fn new(
        renderer: &'a dyn Renderer,
        scene: &'a dyn EntityScene,
        rng: &'a mut dyn RngCore,
        photons: &'a mut Vec<Photon>,
    ) -> Self {
        Self {
            renderer,
            scene,
            rng,
            photons,
        }
    }

    pub fn rng(&mut self) -> &mut &'a mut dyn RngCore {
        &mut self.rng
    }

    pub fn photons(&mut self) -> &mut &'a mut Vec<Photon> {
        &mut self.photons
    }
}
