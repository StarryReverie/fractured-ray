use crate::domain::camera::Resolution;
use crate::domain::color::Color;

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    resolution: Resolution,
    data: Vec<Color>,
}

impl Image {
    pub fn new(resolution: Resolution) -> Self {
        let width = resolution.width() as usize;
        let height = resolution.height() as usize;
        let mut data = Vec::new();
        data.resize(width * height, Color::BLACK);
        Self { resolution, data }
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
            let entry = self
                .data
                .get_mut(row * self.resolution.width() + column)
                .expect("row and column should not be out of bound");
            *entry = color;
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
