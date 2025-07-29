use rand::prelude::*;

use crate::domain::entity::Scene;
use crate::domain::ray::photon::Photon;

use super::Renderer;

pub struct RtContext<'a> {
    renderer: &'a dyn Renderer,
    scene: &'a dyn Scene,
    rng: &'a mut dyn RngCore,
}

impl<'a> RtContext<'a> {
    pub fn new(renderer: &'a dyn Renderer, scene: &'a dyn Scene, rng: &'a mut dyn RngCore) -> Self {
        Self {
            renderer,
            scene,
            rng,
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
