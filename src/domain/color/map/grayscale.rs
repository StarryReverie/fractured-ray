use crate::domain::color::core::Spectrum;
use crate::domain::math::numeric::Val;

use super::Colormap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrayscaleColormap {}

impl GrayscaleColormap {
    #[inline]
    pub fn new() -> Self {
        Self {}
    }
}

impl Colormap for GrayscaleColormap {
    #[inline]
    fn lookup(&self, value: Val) -> Spectrum {
        let value = value.clamp(Val(0.0), Val(1.0));
        Spectrum::broadcast(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::numeric::Val;

    use super::*;

    #[test]
    fn grayscale_colormap_lookup_various_cases() {
        let cmap = GrayscaleColormap::new();

        assert_eq!(cmap.lookup(Val(-1.0)), Spectrum::broadcast(Val(0.0)));
        assert_eq!(cmap.lookup(Val(0.0)), Spectrum::broadcast(Val(0.0)));
        assert_eq!(cmap.lookup(Val(0.5)), Spectrum::broadcast(Val(0.5)));
        assert_eq!(cmap.lookup(Val(1.0)), Spectrum::broadcast(Val(1.0)));
        assert_eq!(cmap.lookup(Val(2.0)), Spectrum::broadcast(Val(1.0)));
    }
}
