use goo_format::PreviewImage;
use image::{imageops::FilterType, RgbaImage};
use nalgebra::Vector2;
use parking_lot::MappedMutexGuard;

use common::{misc::SliceResult, serde::Serializer};

pub enum FormatSliceResult<'a> {
    Goo(SliceResult<'a, goo_format::LayerContent>),
}

pub enum FormatSliceFile {
    Goo(goo_format::File),
}

pub struct SliceInfo {
    pub layers: u32,
    pub resolution: Vector2<u32>,
}

// TODO: convert to dynamic dispatch

impl FormatSliceFile {
    pub fn from_slice_result(
        preview_image: MappedMutexGuard<'_, RgbaImage>,
        slice_result: FormatSliceResult,
    ) -> Self {
        match slice_result {
            FormatSliceResult::Goo(result) => {
                let mut file = goo_format::File::from_slice_result(result);
                file.header.big_preview =
                    PreviewImage::from_image_scaled(&preview_image, FilterType::Nearest);
                file.header.small_preview =
                    PreviewImage::from_image_scaled(&preview_image, FilterType::Nearest);
                Self::Goo(file)
            }
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        match self {
            FormatSliceFile::Goo(file) => file.serialize(ser),
        }
    }

    pub fn info(&self) -> SliceInfo {
        match self {
            FormatSliceFile::Goo(file) => SliceInfo {
                layers: file.header.layer_count,
                resolution: Vector2::new(
                    file.header.x_resolution as u32,
                    file.header.y_resolution as u32,
                ),
            },
        }
    }

    pub fn decode_layer(&self, layer: usize, image: &mut [u8]) {
        match self {
            FormatSliceFile::Goo(file) => {
                let layer_data = &file.layers[layer].data;
                let decoder = goo_format::LayerDecoder::new(layer_data);

                let mut pixel = 0;
                for run in decoder {
                    for _ in 0..run.length {
                        image[pixel] = run.value;
                        pixel += 1;
                    }
                }
            }
        }
    }
}
