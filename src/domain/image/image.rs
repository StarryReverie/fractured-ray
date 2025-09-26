use getset::Getters;

use crate::domain::camera::Resolution;
use crate::domain::color::core::Spectrum;
use crate::domain::math::numeric::Val;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct Image {
    #[getset(get = "pub")]
    resolution: Resolution,
    data: Vec<Spectrum>,
}

impl Image {
    pub fn new(resolution: Resolution) -> Self {
        let len = resolution.width() * resolution.height();
        let data = vec![Spectrum::zero(); len];
        Self { resolution, data }
    }

    #[inline]
    pub fn get(&self, row: usize, column: usize) -> Option<Spectrum> {
        self.locate_index(row, column).map(|index| self.data[index])
    }

    #[inline]
    pub fn get_mut(&mut self, row: usize, column: usize) -> Option<&mut Spectrum> {
        self.locate_index(row, column)
            .and_then(|index| self.data.get_mut(index))
    }

    pub fn set(&mut self, row: usize, column: usize, color: Spectrum) -> bool {
        if let Some(index) = self.locate_index(row, column) {
            self.data[index] = color;
            true
        } else {
            false
        }
    }

    fn locate_index(&self, row: usize, column: usize) -> Option<usize> {
        if !(0..self.resolution.height()).contains(&row) {
            return None;
        }
        if !(0..self.resolution.width()).contains(&column) {
            return None;
        }
        Some(row * self.resolution.width() + column)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageAccumulator {
    image: Image,
    count: Vec<usize>,
}

impl ImageAccumulator {
    pub fn new(image: Image) -> Self {
        let count = vec![1; image.resolution().width() * image.resolution().height()];
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
        if let Some(index) = self.image.locate_index(row, column) {
            let count = self.count.get_mut(index).unwrap();
            let entry = self.image.get_mut(row, column).unwrap();
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
