use crate::domain::color::core::{Color, Spectrum};
use crate::domain::math::numeric::Val;

use super::Colormap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradientColormap {
    start: Spectrum,
    end: Spectrum,
}

impl GradientColormap {
    #[inline]
    pub fn new<SS, SE>(start: SS, end: SE) -> Self
    where
        SS: Into<Spectrum>,
        SE: Into<Spectrum>,
    {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

impl Colormap for GradientColormap {
    #[inline]
    fn lookup(&self, value: Val) -> Spectrum {
        let value = value.clamp(Val(0.0), Val(1.0));
        Spectrum::lerp(self.start, self.end, value)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::numeric::Val;

    use super::*;

    #[test]
    fn gradient_colormap_lookup_succeeds() {
        let c0 = Spectrum::broadcast(Val(0.0));
        let c1 = Spectrum::broadcast(Val(1.0));
        let grad = GradientColormap::new(c0, c1);

        assert_eq!(grad.lookup(Val(-1.0)), c0);
        assert_eq!(grad.lookup(Val(0.0)), c0);
        assert_eq!(grad.lookup(Val(0.5)), Spectrum::lerp(c0, c1, Val(0.5)));
        assert_eq!(grad.lookup(Val(1.0)), c1);
        assert_eq!(grad.lookup(Val(2.0)), c1);
    }
}
