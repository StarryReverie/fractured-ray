mod renderer;

#[cfg(test)]
pub(crate) use renderer::MockRenderer;
pub use renderer::{Configuration, ConfigurationError, CoreRenderer, Renderer};
