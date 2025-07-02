use std::io::{BufWriter, Result as IoResult, Write};

use png::{BitDepth, ColorType, Encoder, SrgbRenderingIntent};

use crate::domain::color::ExternalColor;
use crate::domain::image::Image;

#[derive(Debug)]
pub struct PngWriter<W>
where
    W: Write,
{
    writer: BufWriter<W>,
}

impl<W> PngWriter<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
        }
    }

    pub fn write(self, image: Image) -> IoResult<()> {
        let height = image.resolution().height() as u32;
        let width = image.resolution().width() as u32;
        let data = Self::convert_image(image);

        let mut encoder = Encoder::new(self.writer, width, height);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);
        encoder.set_source_srgb(SrgbRenderingIntent::RelativeColorimetric);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&data)?;

        Ok(())
    }

    fn convert_image(image: Image) -> Vec<u8> {
        let height = image.resolution().height();
        let width = image.resolution().width();
        let mut data = Vec::with_capacity(height * width * 3);

        for row in 0..height {
            for column in 0..width {
                let color = image
                    .get(row, column)
                    .expect("row and column should not be out of bound");
                let color = ExternalColor::from(color);
                data.push(color.red());
                data.push(color.green());
                data.push(color.blue());
            }
        }

        data
    }
}
