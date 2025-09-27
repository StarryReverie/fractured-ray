use std::sync::Arc;

use crate::domain::color::core::{Color, Spectrum};
use crate::domain::image::core::Image;
use crate::domain::math::numeric::Val;
use crate::domain::ray::event::RayIntersection;
use crate::domain::texture::def::{Texture, TextureKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageMap {
    image: Arc<Image>,
}

impl ImageMap {
    #[inline]
    pub fn new<I>(image: I) -> Self
    where
        I: Into<Arc<Image>>,
    {
        let image = image.into();
        Self { image }
    }
}

impl Texture for ImageMap {
    fn kind(&self) -> TextureKind {
        TextureKind::ImageMap
    }

    fn lookup(&self, intersection: &RayIntersection) -> Spectrum {
        let uv = (intersection.uv()).expect("`ImageMap` expects a UV coordinate to be provided");

        let height = self.image.resolution().height() - 1;
        let width = self.image.resolution().width() - 1;

        let r = (Val(1.0) - uv.v()) * Val::from(height);
        let c = uv.u() * Val::from(width);

        let (ri, rf) = (usize::from(r.trunc()), r.fract());
        let (ci, cf) = (usize::from(c.trunc()), c.fract());

        let (r0, r1) = (ri, (ri + 1).min(height));
        let (c0, c1) = (ci, (ci + 1).min(width));

        let s00 = self.image.get(r0, c0).unwrap();
        let s01 = self.image.get(r0, c1).unwrap();
        let s10 = self.image.get(r1, c0).unwrap();
        let s11 = self.image.get(r1, c1).unwrap();

        Spectrum::lerp(
            Spectrum::lerp(s00, s01, cf),
            Spectrum::lerp(s10, s11, cf),
            rf,
        )
    }
}
