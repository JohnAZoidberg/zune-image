#![cfg(feature = "metadata")]

use log::{error, trace};

use crate::image::Image;
use crate::metadata::ImageMetadata;

impl ImageMetadata
{
    /// Parse raw Exif and store it as a field in the data
    ///
    /// Data should point to the first exif byte
    ///
    /// This requires the `metadata` feature to be activated
    /// otherwise it's a compile error
    pub fn parse_raw_exif(&mut self, data: &[u8])
    {
        trace!("Parsing exif data");

        match exif::parse_exif(data)
        {
            Ok(exif) =>
            {
                self.exif = Some(exif.0);
            }
            Err(exif) =>
            {
                error!("Error while parsing exif chunk {:?}", exif)
            }
        };
    }
}

impl Image
{
    /// Change the orientation
    pub fn auto_orient(&mut self) {}
}
