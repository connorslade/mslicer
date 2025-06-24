use goo_format::PreviewImage;
use image::{imageops::FilterType, GrayImage, RgbaImage};
use iter::SliceLayerIterator;
use nalgebra::{Vector2, Vector3};
use parking_lot::MappedMutexGuard;

use common::{image::Image, misc::SliceResult, serde::Serializer};

pub mod iter;
pub mod svg;

pub enum FormatSliceResult<'a> {
    Goo(SliceResult<'a, goo_format::LayerContent>),
}

// todo: replace with trait obj?
pub enum FormatSliceFile {
    Goo(goo_format::File),
    Svg(svg::SvgFile),
}

pub struct SliceInfo {
    pub layers: u32,
    pub resolution: Vector2<u32>,
    pub size: Vector3<f32>,

    pub bottom_layers: u32,
}

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
            FormatSliceFile::Svg(file) => file.serialize(ser),
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
                size: Vector3::new(file.header.x_size, file.header.y_size, file.header.x_size),

                bottom_layers: file.header.bottom_layers,
            },
            FormatSliceFile::Svg(file) => SliceInfo {
                layers: file.layer_count(),

                // todo: actually implement
                resolution: Vector2::zeros(),
                size: Vector3::zeros(),
                bottom_layers: 0,
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
                    let length = run.length as usize;
                    image[pixel..(pixel + length)].fill(run.value);
                    pixel += length;
                }
            }
            FormatSliceFile::Svg(file) => {
                // todo: rasterize svg??
            }
        }
    }
}

impl FormatSliceFile {
    pub fn iter_mut_layers(&mut self) -> SliceLayerIterator<'_> {
        let layers = self.info().layers as usize;
        SliceLayerIterator {
            file: self,
            layer: 0,
            layers,
        }
    }

    pub fn read_layer(&self, layer: usize) -> GrayImage {
        let info = self.info();
        let (width, height) = (info.resolution.x, info.resolution.y);

        let mut raw = vec![0; width as usize * height as usize];
        self.decode_layer(layer, &mut raw);
        GrayImage::from_raw(width, height, raw).unwrap()
    }

    pub fn overwrite_layer(&mut self, layer: usize, image: GrayImage) {
        let info = self.info();
        let (width, height) = (info.resolution.x as usize, info.resolution.y as usize);

        match self {
            FormatSliceFile::Goo(file) => {
                let image = Image::from_raw(width, height, image.into_raw());
                let mut new_layer = goo_format::LayerEncoder::new();
                for run in image.runs() {
                    new_layer.add_run(run.length, run.value)
                }

                let (data, checksum) = new_layer.finish();
                let layer = &mut file.layers[layer];
                layer.data = data;
                layer.checksum = checksum;
            }
            FormatSliceFile::Svg(file) => {
                unimplemented!()
            }
        }
    }
}
