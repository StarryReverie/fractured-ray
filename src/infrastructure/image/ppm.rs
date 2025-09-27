use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, ErrorKind as IoErrorKind, Read, Write};
use std::path::PathBuf;

use snafu::prelude::*;

use crate::domain::camera::Resolution;
use crate::domain::color::external::SRgbColor;
use crate::domain::image::core::Image;
use crate::domain::image::external::*;

#[derive(Debug, Clone)]
pub struct PpmImageResource {
    path: PathBuf,
}

impl PpmImageResource {
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self { path: path.into() }
    }

    fn open_and_read_file(&self) -> Result<Vec<u8>, LoadImageError> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                IoErrorKind::NotFound => {
                    return NotFoundLoadSnafu {
                        path: self.path.clone(),
                    }
                    .fail();
                }
                _ => {
                    return Err(err).context(IoLoadSnafu {
                        path: self.path.clone(),
                    });
                }
            },
        };

        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).context(IoLoadSnafu {
            path: self.path.as_path(),
        })?;
        Ok(buffer)
    }

    fn parse_ppm_format<B>(&self, reader: &mut B) -> Result<(), LoadImageError>
    where
        B: BufRead,
    {
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        ensure_whatever!(
            header.trim() == "P3",
            "`{}` uses unsupported PPM format ({}) other than P3",
            self.path.display(),
            header.trim()
        );
        Ok(())
    }

    fn parse_dimensions<B>(&self, cursor: &mut B) -> Result<(usize, usize), LoadImageError>
    where
        B: BufRead,
    {
        let mut dims = String::new();
        cursor.read_line(&mut dims).unwrap();
        let dims = (dims.split_whitespace())
            .filter_map(|s| s.parse().ok())
            .collect::<Vec<usize>>();
        ensure_whatever!(
            dims.len() == 2,
            "`{}` contains invalid PPM dimensions",
            self.path.display()
        );
        Ok((dims[0], dims[1]))
    }

    fn parse_max_color<B>(&self, cursor: &mut B) -> usize
    where
        B: BufRead,
    {
        let mut max_color = String::new();
        cursor.read_line(&mut max_color).unwrap();
        max_color.trim().parse::<usize>().unwrap_or(255)
    }

    fn parse_pixel_data<B>(
        &self,
        cursor: &mut B,
        width: usize,
        height: usize,
    ) -> Result<Vec<u8>, LoadImageError>
    where
        B: BufRead,
    {
        let mut pixel_lines = String::new();
        cursor.read_to_string(&mut pixel_lines).unwrap();
        let pixels = (pixel_lines.split_whitespace())
            .filter_map(|v| v.parse::<u8>().ok())
            .collect::<Vec<_>>();
        ensure_whatever!(
            pixels.len() == width * height * 3,
            "`{}`'s pixel data size mismatched",
            self.path.display()
        );
        Ok(pixels)
    }

    fn create_and_write_file(&self, buffer: Vec<u8>) -> Result<(), SaveImageError> {
        let mut file = File::create(&self.path).context(IoSaveSnafu {
            path: self.path.clone(),
        })?;
        file.write_all(&buffer).context(IoSaveSnafu {
            path: self.path.clone(),
        })?;
        Ok(())
    }
}

impl ImageResource for PpmImageResource {
    fn load(&self) -> Result<Image, LoadImageError> {
        let buffer = self.open_and_read_file()?;
        let mut cursor = Cursor::new(buffer);

        self.parse_ppm_format(&mut cursor)?;
        let (width, height) = self.parse_dimensions(&mut cursor)?;
        let max_color = self.parse_max_color(&mut cursor);
        let pixels = self.parse_pixel_data(&mut cursor, width, height)?;

        let mut image = Image::new(Resolution::new(height, (width, height)).unwrap());
        for row in 0..height {
            for col in 0..width {
                let idx = (row * width + col) * 3;
                let red = (pixels[idx] as f32 / max_color as f32 * 255.0).floor() as u8;
                let green = (pixels[idx + 1] as f32 / max_color as f32 * 255.0).floor() as u8;
                let blue = (pixels[idx + 2] as f32 / max_color as f32 * 255.0).floor() as u8;
                image.set(row, col, SRgbColor::new(red, green, blue).into());
            }
        }

        Ok(image)
    }

    fn save(&self, image: &Image) -> Result<(), SaveImageError> {
        let mut buffer = Vec::new();

        let width = image.resolution().width();
        let height = image.resolution().height();

        writeln!(buffer, "P3").unwrap();
        writeln!(buffer, "{} {}", width, height).unwrap();
        writeln!(buffer, "255").unwrap();

        for row in 0..height {
            for column in 0..width {
                let color = SRgbColor::from(image.get(row, column).unwrap());
                let (r, g, b) = (color.red(), color.green(), color.blue());
                write!(buffer, "{} {} {} ", r, g, b).unwrap();
            }
            writeln!(buffer).unwrap();
        }

        self.create_and_write_file(buffer)?;
        Ok(())
    }
}
