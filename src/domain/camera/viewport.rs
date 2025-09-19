use getset::{CopyGetters, Getters};
use snafu::prelude::*;

use crate::domain::math::geometry::Distance;
use crate::domain::math::numeric::Val;

use super::Resolution;

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct Viewport {
    #[getset(get = "pub")]
    resolution: Resolution,
    #[getset(get_copy = "pub")]
    width: Distance,
    #[getset(get_copy = "pub")]
    height: Distance,
    #[getset(get_copy = "pub")]
    pixel_size: Val,
}

impl Viewport {
    pub fn new(resolution: Resolution, height: Distance) -> Self {
        let aspect_ratio = Val::from(resolution.width()) / Val::from(resolution.height());
        let width = Distance::new(height.value() * aspect_ratio).unwrap();
        let pixel_size = height.value() / Val::from(resolution.height());

        Self {
            resolution,
            width,
            height,
            pixel_size,
        }
    }

    pub fn index_to_percentage(
        &self,
        row: usize,
        column: usize,
        offset: Offset,
    ) -> Option<(Val, Val)> {
        if self.contains_index(row, column) {
            Some((
                (Val::from(row) + offset.row()) / Val::from(self.resolution.height()),
                (Val::from(column) + offset.column()) / Val::from(self.resolution.width()),
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offset {
    row: Val,
    column: Val,
}

impl Offset {
    pub fn new(row: Val, column: Val) -> Result<Self, TryNewOffsetError> {
        ensure!(
            (Val(0.0)..=Val(1.0)).contains(&row) && (Val(0.0)..=Val(1.0)).contains(&column),
            InvalidOffsetSnafu
        );
        Ok(Self { row, column })
    }

    pub fn center() -> Self {
        Self {
            row: Val(0.5),
            column: Val(0.5),
        }
    }

    pub fn row(&self) -> Val {
        self.row
    }

    pub fn column(&self) -> Val {
        self.column
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
pub enum TryNewOffsetError {
    #[snafu(display("offset is out of range [0, 1]"))]
    InvalidOffset,
}
