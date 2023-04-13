#![cfg(feature = "metadata")]

use log::warn;
use zune_core::bit_depth::BitType;

use crate::errors::ImageErrors;
use crate::image::Image;
use crate::impls::flip::Flip;
use crate::impls::flop::Flop;
use crate::impls::rotate::Rotate;
use crate::impls::transpose::Transpose;
use crate::traits::OperationsTrait;

pub enum OrientationType
{
    DoNothing = 1,
    FlipHorizontally = 2,
    Rotate180 = 3,
    FlipVertically = 4
}

pub struct AutoOrient;

impl OperationsTrait for AutoOrient
{
    fn get_name(&self) -> &'static str
    {
        "Auto orient"
    }

    fn execute_impl(&self, image: &mut Image) -> Result<(), ImageErrors>
    {
        // check if we have exif orientation metadata and transform it
        // to be this orientation
        #[cfg(feature = "metadata")]
        {
            use exif::{Tag, Value};

            if let Some(data) = image.metadata.exif.clone()
            {
                for field in data
                {
                    // look for the orientation tag
                    if field.tag == Tag::Orientation
                    {
                        match &field.value
                        {
                            Value::Short(bytes) =>
                            {
                                if bytes.is_empty()
                                {
                                    warn!("The exif value is empty, cannot orient");
                                    return Ok(());
                                }
                                match bytes[0]
                                {
                                    1 => (), // orientation is okay
                                    2 =>
                                    {
                                        Flop::new().execute_impl(image)?;
                                    }

                                    3 =>
                                    {
                                        Flip::new().execute_impl(image)?;
                                    }
                                    4 =>
                                    {
                                        // swap top with bottom
                                        // 180 degree rotation
                                        Rotate::new(180.0).execute_impl(image)?;
                                    }
                                    5 =>
                                    {
                                        Transpose::new().execute_impl(image)?;
                                    }
                                    6 =>
                                    {
                                        Transpose::new().execute_impl(image)?;
                                        Flop::new().execute_impl(image)?;
                                    }
                                    7 =>
                                    {
                                        Transpose::new().execute_impl(image)?;
                                        Flip::new().execute_impl(image)?;
                                    }
                                    8 =>
                                    {
                                        Transpose::new().execute_impl(image)?;
                                        Rotate::new(180.0).execute_impl(image)?;
                                    }

                                    _ =>
                                    {
                                        warn!(
                                            "Unknown exif orientation tag {:?}, ignoring it",
                                            &field.value
                                        );
                                    }
                                }
                            }
                            _ =>
                            {
                                warn!("Invalid exif orientation type, ignoring it");
                            }
                        }
                    }
                }
            }
            // update exif
            if let Some(data) = &mut image.metadata.exif
            {
                for field in data
                {
                    // set orientation to do nothing
                    if field.tag == Tag::Orientation
                    {
                        field.value = Value::Byte(vec![1]);
                    }
                }
            }
        }
        Ok(())
    }

    fn supported_types(&self) -> &'static [BitType]
    {
        &[BitType::U16, BitType::U8]
    }
}
