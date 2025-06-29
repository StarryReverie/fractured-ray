use snafu::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    width: usize,
    height: usize,
}

impl Resolution {
    pub fn new(height: usize, aspect_ratio: (usize, usize)) -> Result<Self, TryNewResolutionError> {
        ensure!(height > 0, InvalidHeightSnafu);
        ensure!(
            aspect_ratio.0 != 0 && aspect_ratio.1 != 0,
            InvalidAspectRatioSnafu,
        );

        ensure!(
            height * aspect_ratio.0 % aspect_ratio.1 == 0,
            NonIntegerWidthSnafu
        );

        Ok(Self {
            width: height * aspect_ratio.0 / aspect_ratio.1,
            height,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewResolutionError {
    #[snafu(display("height is not positive"))]
    InvalidHeight,
    #[snafu(display("aspect ratio has a zero-valued component"))]
    InvalidAspectRatio,
    #[snafu(display("width is not an integer given the corresponding height"))]
    NonIntegerWidth,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_new_succeeds() {
        let resolution = Resolution::new(1440, (16, 9)).unwrap();
        assert_eq!(resolution.width, 2560);
    }

    #[test]
    fn resolution_new_fails_when_height_is_invalid() {
        assert_eq!(
            Resolution::new(0, (16, 9)),
            Err(TryNewResolutionError::InvalidHeight),
        );
    }

    #[test]
    fn resolution_new_fails_when_aspect_ratio_is_invalid() {
        assert_eq!(
            Resolution::new(1, (0, 1)),
            Err(TryNewResolutionError::InvalidAspectRatio),
        );
    }

    #[test]
    fn resolution_new_fails_when_width_is_non_integer() {
        assert_eq!(
            Resolution::new(768, (16, 10)),
            Err(TryNewResolutionError::NonIntegerWidth)
        );
    }
}
