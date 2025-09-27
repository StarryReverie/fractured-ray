use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind as IoErrorKind};
use std::path::PathBuf;

use png::{BitDepth, ColorType, Decoder, Encoder, OutputInfo, SrgbRenderingIntent};
use snafu::prelude::*;

use crate::domain::camera::Resolution;
use crate::domain::color::external::SRgbColor;
use crate::domain::image::core::Image;
use crate::domain::image::external::*;

#[derive(Debug, Clone)]
pub struct PngImageResource {
    path: PathBuf,
}

impl PngImageResource {
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self { path: path.into() }
    }

    fn convert_image(image: &Image) -> Vec<u8> {
        let height = image.resolution().height();
        let width = image.resolution().width();
        let mut data = Vec::with_capacity(height * width * 3);

        for row in 0..height {
            for column in 0..width {
                let color = image.get(row, column).unwrap();
                let color = SRgbColor::from(color);
                data.push(color.red());
                data.push(color.green());
                data.push(color.blue());
            }
        }

        data
    }

    fn open_file_for_load(&self) -> Result<File, LoadImageError> {
        match File::open(&self.path) {
            Ok(file) => Ok(file),
            Err(err) => match err.kind() {
                IoErrorKind::NotFound => NotFoundLoadSnafu {
                    path: self.path.as_path(),
                }
                .fail(),
                _ => Err(err).context(IoLoadSnafu {
                    path: self.path.as_path(),
                }),
            },
        }
    }

    fn decode_png_metadata(&self, file: File) -> Result<(OutputInfo, Vec<u8>), LoadImageError> {
        let decoder = Decoder::new(BufReader::new(file));
        let mut reader = whatever!(
            decoder.read_info(),
            "could not read metadata from `{}`",
            self.path.display()
        );

        let mut buf = vec![0; reader.output_buffer_size()];
        let info = whatever!(
            reader.next_frame(&mut buf),
            "could not read data from `{}`",
            self.path.display()
        );

        ensure_whatever!(
            matches!(
                info.color_type,
                ColorType::Grayscale | ColorType::GrayscaleAlpha | ColorType::Rgb | ColorType::Rgba
            ) && info.bit_depth == BitDepth::Eight,
            "`{}` provides an unsupported color type",
            self.path.display()
        );

        Ok((info, buf))
    }

    fn decode_image(&self, info: OutputInfo, buf: &[u8]) -> Result<Image, LoadImageError> {
        let width = info.width as usize;
        let height = info.height as usize;
        let mut image = Image::new(Resolution::new(height, (width, height)).unwrap());
        let bytes = &buf[..info.buffer_size()];

        for row in 0..height {
            for column in 0..width {
                let color = match info.color_type {
                    ColorType::Grayscale => {
                        let idx = row * width + column;
                        SRgbColor::new(bytes[idx], bytes[idx], bytes[idx])
                    }
                    ColorType::GrayscaleAlpha => {
                        let idx = (row * width + column) * 2;
                        SRgbColor::new(bytes[idx], bytes[idx], bytes[idx])
                    }
                    ColorType::Rgb => {
                        let idx = (row * width + column) * 3;
                        SRgbColor::new(bytes[idx], bytes[idx + 1], bytes[idx + 2])
                    }
                    ColorType::Rgba => {
                        let idx = (row * width + column) * 4;
                        SRgbColor::new(bytes[idx], bytes[idx + 1], bytes[idx + 2])
                    }
                    _ => unreachable!("other unsupported color types should be checked"),
                };
                image.set(row, column, color.into());
            }
        }

        Ok(image)
    }

    fn open_file_for_save(&self) -> Result<File, SaveImageError> {
        File::create(&self.path).context(IoSaveSnafu {
            path: self.path.clone(),
        })
    }

    fn encode_png(&self, image: &Image, file: File) -> Result<(), SaveImageError> {
        let height = image.resolution().height() as u32;
        let width = image.resolution().width() as u32;
        let mut encoder = Encoder::new(BufWriter::new(file), width, height);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);
        encoder.set_source_srgb(SrgbRenderingIntent::RelativeColorimetric);

        let mut writer = whatever!(
            encoder.write_header(),
            "could not write metadata to `{}`",
            self.path.display()
        );
        let data = Self::convert_image(image);
        whatever!(
            writer.write_image_data(&data),
            "could not write all data to `{}`",
            self.path.display()
        );
        Ok(())
    }
}

impl ImageResource for PngImageResource {
    fn load(&self) -> Result<Image, LoadImageError> {
        let file = self.open_file_for_load()?;
        let (info, buf) = self.decode_png_metadata(file)?;
        self.decode_image(info, &buf)
    }

    fn save(&self, image: &Image) -> Result<(), SaveImageError> {
        let file = self.open_file_for_save()?;
        self.encode_png(image, file)
    }
}
