use rand::prelude::*;

use crate::domain::entity::Scene;
use crate::domain::ray::photon::{Photon, PhotonMap, SearchPolicy};

use super::{Configuration, Renderer};

pub struct RtContext<'a> {
    renderer: &'a dyn Renderer,
    scene: &'a dyn Scene,
    rng: &'a mut dyn RngCore,
    config: &'a Configuration,
    photon_global: PhotonInfo<'a>,
    photon_casutic: PhotonInfo<'a>,
}

impl<'a> RtContext<'a> {
    pub fn new(
        renderer: &'a dyn Renderer,
        scene: &'a dyn Scene,
        rng: &'a mut dyn RngCore,
        config: &'a Configuration,
        photon_global: PhotonInfo<'a>,
        photon_casutic: PhotonInfo<'a>,
    ) -> Self {
        Self {
            renderer,
            scene,
            rng,
            config,
            photon_global,
            photon_casutic,
        }
    }

    pub fn renderer(&self) -> &'a (dyn Renderer + 'static) {
        self.renderer
    }

    pub fn scene(&self) -> &'a (dyn Scene + 'static) {
        self.scene
    }

    pub fn rng(&mut self) -> &mut &'a mut dyn RngCore {
        &mut self.rng
    }

    pub fn config(&self) -> &'a Configuration {
        self.config
    }

    pub fn photon_global(&self) -> &PhotonInfo<'a> {
        &self.photon_global
    }

    pub fn photon_casutic(&self) -> &PhotonInfo<'a> {
        &self.photon_casutic
    }
}

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

    pub fn photons(&self) -> &'a PhotonMap {
        self.photons
    }

    pub fn policy(&self) -> SearchPolicy {
        self.policy
    }

    pub fn emitted(&self) -> usize {
        self.emitted
    }
}

pub struct PmContext<'a> {
    renderer: &'a dyn Renderer,
    scene: &'a dyn Scene,
    rng: &'a mut dyn RngCore,
    photons: &'a mut Vec<Photon>,
}

impl<'a> PmContext<'a> {
    pub fn new(
        renderer: &'a dyn Renderer,
        scene: &'a dyn Scene,
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

    pub fn renderer(&self) -> &'a (dyn Renderer + 'static) {
        self.renderer
    }

    pub fn scene(&self) -> &'a (dyn Scene + 'static) {
        self.scene
    }

    pub fn rng(&mut self) -> &mut &'a mut dyn RngCore {
        &mut self.rng
    }

    pub fn photons(&mut self) -> &mut &'a mut Vec<Photon> {
        &mut self.photons
    }
}
