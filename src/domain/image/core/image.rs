use getset::Getters;

use crate::domain::camera::Resolution;
use crate::domain::color::core::Spectrum;
use crate::domain::math::numeric::Val;

use super::BlockedArray;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct Image {
    #[getset(get = "pub")]
    resolution: Resolution,
    data: BlockedArray<Spectrum>,
}

impl Image {
    const IMAGE_BLOCK_LOG2_SIZE: usize = 3;

    pub fn new(resolution: Resolution) -> Self {
        let data = BlockedArray::new(
            resolution.height(),
            resolution.width(),
            Self::IMAGE_BLOCK_LOG2_SIZE,
        );
        Self { resolution, data }
    }

    #[inline]
    pub fn get(&self, row: usize, column: usize) -> Option<Spectrum> {
        self.data.get(row, column).cloned()
    }

    #[inline]
    pub fn get_mut(&mut self, row: usize, column: usize) -> Option<&mut Spectrum> {
        self.data.get_mut(row, column)
    }

    #[inline]
    pub fn set(&mut self, row: usize, column: usize, color: Spectrum) -> bool {
        self.data.set(row, column, color)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageAccumulator {
    image: Image,
    count: BlockedArray<usize>,
}

impl ImageAccumulator {
    pub fn new(image: Image) -> Self {
        let count = BlockedArray::new(
            image.resolution().height(),
            image.resolution().width(),
            Image::IMAGE_BLOCK_LOG2_SIZE,
        );
        Self { image, count }
    }

    #[inline]
    pub fn resolution(&self) -> &Resolution {
        self.image.resolution()
    }

    #[inline]
    pub fn get(&self, row: usize, column: usize) -> Option<Spectrum> {
        self.image.get(row, column)
    }

    pub fn record(&mut self, row: usize, column: usize, color: Spectrum) -> bool {
        if let (Some(count), Some(entry)) = (
            self.count.get_mut(row, column),
            self.image.get_mut(row, column),
        ) {
            *entry = *entry * (Val::from(*count) / (Val::from(*count) + Val::from(1.0)))
                + color * (Val::from(1.0) / (Val::from(*count) + Val::from(1.0)));
            *count += 1;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn into_inner(self) -> Image {
        self.image
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::camera::Resolution;
    use crate::domain::color::core::Spectrum;
    use crate::domain::math::numeric::Val;

    use super::*;

    #[test]
    fn image_get_and_set_succeeds() {
        let res = Resolution::new(600, (4, 3)).unwrap();
        let mut img = Image::new(res);

        let c1 = Spectrum::broadcast(Val(1.0));
        let c2 = Spectrum::broadcast(Val(0.5));

        assert!(img.set(200, 300, c1));
        assert_eq!(img.get(200, 300), Some(c1));

        assert!(img.set(0, 0, c2));
        assert_eq!(img.get(0, 0), Some(c2));
    }

    #[test]
    fn image_get_and_set_fails_when_out_of_bounds() {
        let res = Resolution::new(600, (4, 3)).unwrap();
        let img = Image::new(res);

        assert_eq!(img.get(600, 0), None);
        assert_eq!(img.get(0, 800), None);
        assert_eq!(img.get(601, 801), None);
    }

    #[test]
    fn image_accumulator_record_succeeds() {
        let res = Resolution::new(10, (10, 1)).unwrap();
        let img = Image::new(res);
        let mut acc = ImageAccumulator::new(img);

        let c1 = Spectrum::broadcast(Val(1.0));
        let c2 = Spectrum::broadcast(Val(0.0));

        assert!(acc.record(5, 5, c1));
        assert_eq!(acc.get(5, 5), Some(c1));

        assert!(acc.record(5, 5, c2));
        assert_eq!(acc.get(5, 5), Some(Spectrum::broadcast(Val(0.5))));

        assert!(acc.record(5, 5, c1));
        assert_eq!(
            acc.get(5, 5),
            Some(Spectrum::broadcast(Val((0.5 * 2.0 + 1.0) / 3.0)))
        );
    }
}
