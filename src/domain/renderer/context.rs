use crate::domain::entity::Scene;

use super::Renderer;

pub struct Context<'a> {
    renderer: &'a dyn Renderer,
    scene: &'a dyn Scene,
}

impl<'a> Context<'a> {
    pub fn new(renderer: &'a dyn Renderer, scene: &'a dyn Scene) -> Self {
        Self { renderer, scene }
    }

    pub fn renderer(&self) -> &'a (dyn Renderer + 'static) {
        self.renderer
    }

    pub fn scene(&self) -> &'a (dyn Scene + 'static) {
        self.scene
    }
}
