use snafu::prelude::*;

use crate::domain::color::core::{Color, Spectrum};
use crate::domain::math::numeric::Val;

use super::Colormap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteColormap {
    colors: Vec<(Spectrum, Val)>,
}

impl PaletteColormap {
    pub fn new<C, S>(colors: C) -> Result<Self, TryNewPaletteColormapError>
    where
        C: IntoIterator<Item = (S, Val)>,
        S: Into<Spectrum>,
    {
        let mut colors = (colors.into_iter())
            .map(|(color, pos)| (color.into(), pos))
            .collect::<Vec<_>>();
        colors.sort_by_key(|(_, pos)| *pos);

        ensure!(colors.len() >= 2, ColorsNotEnoughSnafu);
        for (_, position) in &colors {
            ensure!(
                (Val(0.0)..=Val(1.0)).contains(position),
                PositionOutOfBoundSnafu,
            );
        }
        ensure!(
            colors.first().is_some_and(|(_, pos)| *pos == Val(0.0)),
            InvalidFirstPositonSnafu,
        );
        ensure!(
            colors.last().is_some_and(|(_, pos)| *pos == Val(1.0)),
            InvalidLastPositonSnafu,
        );

        Ok(Self { colors })
    }
}

impl Colormap for PaletteColormap {
    fn lookup(&self, mut value: Val) -> Spectrum {
        value = value.clamp(Val(0.0), Val(1.0));
        for w in self.colors.windows(2) {
            let ((color1, pos1), (color2, pos2)) = (w[0], w[1]);
            if pos1 <= value && value <= pos2 {
                if pos1 != pos2 {
                    let t = (value - pos1) / (pos2 - pos1);
                    return Spectrum::lerp(color1, color2, t);
                } else {
                    return Val(0.5) * (color1 + color2);
                }
            }
        }
        self.colors.last().unwrap().0
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewPaletteColormapError {
    #[snafu(display("palette should have at least two colors"))]
    ColorsNotEnough,
    #[snafu(display("color's position should be in [0, 1]"))]
    PositionOutOfBound,
    #[snafu(display("the first color's position should be 0"))]
    InvalidFirstPositon,
    #[snafu(display("the last color's position should be 1"))]
    InvalidLastPositon,
}

#[cfg(test)]
mod tests {
    use crate::domain::color::core::Spectrum;
    use crate::domain::math::numeric::Val;

    use super::*;

    #[test]
    fn palette_colormap_new_fails() {
        let c0 = Spectrum::broadcast(Val(0.0));
        let c1 = Spectrum::broadcast(Val(1.0));

        assert!(matches!(
            PaletteColormap::new(vec![(c0, Val(0.0))]),
            Err(TryNewPaletteColormapError::ColorsNotEnough)
        ));

        assert!(matches!(
            PaletteColormap::new(vec![(c0, Val(0.1)), (c1, Val(1.0))]),
            Err(TryNewPaletteColormapError::InvalidFirstPositon)
        ));

        assert!(matches!(
            PaletteColormap::new(vec![(c0, Val(0.0)), (c1, Val(0.9))]),
            Err(TryNewPaletteColormapError::InvalidLastPositon)
        ));

        assert!(matches!(
            dbg!(PaletteColormap::new(vec![(c0, Val(-0.1)), (c1, Val(1.0))])),
            Err(TryNewPaletteColormapError::PositionOutOfBound)
        ));
        assert!(matches!(
            PaletteColormap::new(vec![(c0, Val(0.0)), (c1, Val(1.1))]),
            Err(TryNewPaletteColormapError::PositionOutOfBound)
        ));
    }

    #[test]
    fn palette_colormap_lookup_succeeds() {
        let c0 = Spectrum::broadcast(Val(0.0));
        let c1 = Spectrum::broadcast(Val(1.0));
        let palette = PaletteColormap::new(vec![(c0, Val(0.0)), (c1, Val(1.0))]).unwrap();

        assert_eq!(palette.lookup(Val(-1.0)), c0);
        assert_eq!(palette.lookup(Val(0.0)), c0);
        assert_eq!(palette.lookup(Val(0.5)), Spectrum::lerp(c0, c1, Val(0.5)));
        assert_eq!(palette.lookup(Val(1.0)), c1);
        assert_eq!(palette.lookup(Val(2.0)), c1);
    }
}
