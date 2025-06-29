use std::io::{BufWriter, Result as IoResult, Write};

use crate::domain::color::ExternalColor;
use crate::domain::image::Image;

#[derive(Debug)]
pub struct PpmWriter<W>
where
    W: Write,
{
    writer: BufWriter<W>,
}

impl<W> PpmWriter<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
        }
    }

    pub fn write(mut self, image: Image) -> IoResult<()> {
        writeln!(self.writer, "P3")?;
        writeln!(
            self.writer,
            "{} {}",
            image.resolution().width(),
            image.resolution().height()
        )?;
        writeln!(self.writer, "255")?;
        for row in 0..image.resolution().height() {
            for column in 0..image.resolution().width() {
                let color = image
                    .get(row, column)
                    .expect("row and column should not be out of bound");
                let color = ExternalColor::from(color);
                write!(
                    self.writer,
                    "{} {} {} ",
                    color.red(),
                    color.green(),
                    color.blue()
                )?;
            }
            writeln!(self.writer)?;
        }
        Ok(())
    }
}
