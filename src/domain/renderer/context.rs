use super::Renderer;

pub struct Context<'a> {
    renderer: &'a dyn Renderer,
}

impl<'a> Context<'a> {
    pub fn new(renderer: &'a dyn Renderer) -> Self {
        Self { renderer }
    }

    pub fn renderer(&self) -> &'a (dyn Renderer + 'static) {
        self.renderer
    }
}
