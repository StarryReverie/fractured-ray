use snafu::prelude::*;

use super::Resolution;

#[derive(Debug, Clone, PartialEq)]
pub struct Viewport {
    resolution: Resolution,
    width: f32,
    height: f32,
    pixel_size: f32,
}

impl Viewport {
    pub fn new(resolution: Resolution, height: f32) -> Result<Self, TryNewViewportError> {
        ensure!(height > 0.0, InvalidHeightSnafu);

        let aspect_ratio = (resolution.width() as f32) / (resolution.height() as f32);
        let width = height * aspect_ratio;
        let pixel_size = height / (resolution.height() as f32);

        Ok(Self {
            resolution,
            width,
            height,
            pixel_size,
        })
    }

    pub fn resolution(&self) -> &Resolution {
        &self.resolution
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn pixel_size(&self) -> f32 {
        self.pixel_size
    }

    pub fn index_to_percentage(
        &self,
        row: usize,
        column: usize,
        offset: Offset,
    ) -> Option<(f32, f32)> {
        if self.contains_index(row, column) {
            Some((
                (row as f32 + offset.row()) / (self.resolution.height() as f32),
                (column as f32 + offset.column()) / (self.resolution.width() as f32),
            ))
        } else {
            None
        }
    }

    fn contains_index(&self, row: usize, column: usize) -> bool {
        (0..self.resolution.height()).contains(&row)
            && (0..self.resolution.width()).contains(&column)
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewViewportError {
    #[snafu(display("viewport height is not positive"))]
    InvalidHeight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offset {
    row: f32,
    column: f32,
}

impl Offset {
    pub fn new(row: f32, column: f32) -> Result<Self, TryNewOffsetError> {
        ensure!(
            (0.0..=1.0).contains(&row) && (0.0..=1.0).contains(&column),
            InvalidOffsetSnafu
        );
        Ok(Self { row, column })
    }

    pub fn center() -> Self {
        Self {
            row: 0.5,
            column: 0.5,
        }
    }

    pub fn row(&self) -> f32 {
        self.row
    }

    pub fn column(&self) -> f32 {
        self.column
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
pub enum TryNewOffsetError {
    #[snafu(display("offset is out of range [0, 1]"))]
    InvalidOffset,
}
