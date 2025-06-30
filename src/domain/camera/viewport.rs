use snafu::prelude::*;

use super::Resolution;

#[derive(Debug, Clone, PartialEq)]
pub struct Viewport {
    resolution: Resolution,
    width: f64,
    height: f64,
    pixel_size: f64,
}

impl Viewport {
    pub fn new(resolution: Resolution, height: f64) -> Result<Self, TryNewViewportError> {
        ensure!(height > 0.0, InvalidHeightSnafu);

        let aspect_ratio = (resolution.width() as f64) / (resolution.height() as f64);
        let width = height * aspect_ratio;
        let pixel_size = height / (resolution.height() as f64);

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

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn pixel_size(&self) -> f64 {
        self.pixel_size
    }

    pub fn index_to_percentage(
        &self,
        row: usize,
        column: usize,
        offset: Offset,
    ) -> Option<(f64, f64)> {
        if self.contains_index(row, column) {
            Some((
                (row as f64 + offset.row()) / (self.resolution.height() as f64),
                (column as f64 + offset.column()) / (self.resolution.width() as f64),
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
    row: f64,
    column: f64,
}

impl Offset {
    pub fn new(row: f64, column: f64) -> Result<Self, TryNewOffsetError> {
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

    pub fn row(&self) -> f64 {
        self.row
    }

    pub fn column(&self) -> f64 {
        self.column
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
pub enum TryNewOffsetError {
    #[snafu(display("offset is out of range [0, 1]"))]
    InvalidOffset,
}
