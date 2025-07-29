use rand::prelude::*;

use crate::domain::entity::Scene;
use crate::domain::ray::photon::{Photon, PhotonMap};

use super::{Configuration, Renderer};

pub struct RtContext<'a> {
    renderer: &'a dyn Renderer,
    scene: &'a dyn Scene,
    rng: &'a mut dyn RngCore,
    pm_global: &'a PhotonMap,
    pm_caustic: &'a PhotonMap,
    config: &'a Configuration,
}

impl<'a> RtContext<'a> {
    pub fn new(
        renderer: &'a dyn Renderer,
        scene: &'a dyn Scene,
        rng: &'a mut dyn RngCore,
        pm_global: &'a PhotonMap,
        pm_caustic: &'a PhotonMap,
        config: &'a Configuration,
    ) -> Self {
        Self {
            renderer,
            scene,
            rng,
            pm_global,
            pm_caustic,
            config,
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

    pub fn pm_global(&self) -> &'a PhotonMap {
        self.pm_global
    }

    pub fn pm_caustic(&self) -> &'a PhotonMap {
        self.pm_caustic
    }

    pub fn config(&self) -> &'a Configuration {
        self.config
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
