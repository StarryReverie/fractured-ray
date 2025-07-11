use crate::domain::camera::Resolution;
use crate::domain::color::Color;
use crate::domain::math::numeric::Val;

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    resolution: Resolution,
    data: Vec<Color>,
    count: Vec<usize>,
}

impl Image {
    pub fn new(resolution: Resolution) -> Self {
        let width = resolution.width();
        let height = resolution.height();

        let mut data = Vec::new();
        data.resize(width * height, Color::BLACK);

        let mut count = Vec::new();
        count.resize(width * height, 1);

        Self {
            resolution,
            data,
            count,
        }
    }

    pub fn resolution(&self) -> &Resolution {
        &self.resolution
    }

    pub fn get(&self, row: usize, column: usize) -> Option<Color> {
        if self.contains_index(row, column) {
            self.data
                .get(row * self.resolution.width() + column)
                .cloned()
        } else {
            None
        }
    }

    pub fn record(&mut self, row: usize, column: usize, color: Color) -> bool {
        if self.contains_index(row, column) {
            let index = row * self.resolution.width() + column;
            let count = self
                .count
                .get_mut(index)
                .expect("row and column should not be out of bound");
            let entry = self
                .data
                .get_mut(index)
                .expect("row and column should not be out of bound");
            *entry = *entry * (Val::from(*count) / (Val::from(*count) + Val::from(1.0)))
                + color * (Val::from(1.0) / (Val::from(*count) + Val::from(1.0)));
            *count += 1;
            true
        } else {
            false
        }
    }

    fn contains_index(&self, row: usize, column: usize) -> bool {
        (0..self.resolution.height()).contains(&row)
            && (0..self.resolution.width()).contains(&column)
    }
}
