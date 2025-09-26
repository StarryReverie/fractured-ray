mod def;
mod gradient;
mod grayscale;
mod palette;

pub use def::Colormap;
pub use gradient::GradientColormap;
pub use grayscale::GrayscaleColormap;
pub use palette::{PaletteColormap, TryNewPaletteColormapError};
