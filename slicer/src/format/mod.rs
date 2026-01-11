use image::{GrayImage, RgbaImage, imageops::FilterType};
use iter::SliceLayerIterator;
use nalgebra::{Vector2, Vector3};
use parking_lot::MappedMutexGuard;

use common::{
    format::Format,
    image::Image,
    misc::{EncodableLayer, Run, SliceResult, VectorSliceResult},
    serde::Serializer,
};

use crate::format::svg::SvgFile;

pub mod iter;
pub mod svg;

pub enum FormatSliceResult<'a> {
    Goo(SliceResult<'a, goo_format::LayerContent>),
    Ctb(SliceResult<'a, ctb_format::Layer>),
    Svg(VectorSliceResult<'a>),
}

// todo: replace with trait obj?
pub enum FormatSliceFile {
    Goo(Box<goo_format::File>),
    Ctb(Box<ctb_format::File>),
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
        preview: MappedMutexGuard<'_, RgbaImage>,
        slice_result: FormatSliceResult,
    ) -> Self {
        match slice_result {
            FormatSliceResult::Goo(result) => {
                let mut file = goo_format::File::from_slice_result(result);
                file.header.big_preview =
                    goo_format::PreviewImage::from_image_scaled(&preview, FilterType::Nearest);
                file.header.small_preview =
                    goo_format::PreviewImage::from_image_scaled(&preview, FilterType::Nearest);
                Self::Goo(Box::new(file))
            }
            FormatSliceResult::Ctb(result) => {
                let mut file = ctb_format::File::from_slice_result(result);
                file.large_preview = ctb_format::PreviewImage::from_image(&preview);

                let (width, height) = (preview.width() * 3 / 4, preview.height() * 3 / 4);
                let small_preview =
                    image::imageops::resize(&*preview, width, height, FilterType::Nearest);
                file.small_preview = ctb_format::PreviewImage::from_image(&small_preview);

                Self::Ctb(Box::new(file))
            }
            FormatSliceResult::Svg(result) => Self::Svg(SvgFile::new(result)),
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        match self {
            FormatSliceFile::Goo(file) => file.serialize(ser),
            FormatSliceFile::Ctb(file) => file.serialize(ser),
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
            FormatSliceFile::Ctb(file) => SliceInfo {
                layers: file.layers.len() as u32,
                resolution: file.resolution,
                size: file.size,
                bottom_layers: file.bottom_layer_count,
            },
            FormatSliceFile::Svg(file) => SliceInfo {
                layers: file.layer_count(),

                // todo: actually implement
                resolution: Vector2::repeat(1),
                size: Vector3::zeros(),
                bottom_layers: 0,
            },
        }
    }

    pub fn as_format(&self) -> Format {
        match self {
            FormatSliceFile::Goo(_) => Format::Goo,
            FormatSliceFile::Ctb(_) => Format::Ctb,
            FormatSliceFile::Svg(_) => Format::Svg,
        }
    }

    pub fn decode_layer(&self, layer: usize, image: &mut [u8]) {
        fn rle_decode(decoder: impl Iterator<Item = Run>, image: &mut [u8]) {
            let mut pixel = 0;
            for run in decoder {
                let length = run.length as usize;
                image[pixel..(pixel + length)].fill(run.value);
                pixel += length;
            }
        }

        match self {
            FormatSliceFile::Goo(file) => {
                let layer_data = &file.layers[layer].data;
                let decoder = goo_format::LayerDecoder::new(layer_data);
                rle_decode(decoder, image);
            }
            FormatSliceFile::Ctb(file) => {
                let data = &file.layers[layer].data;
                let decoder = ctb_format::LayerDecoder::new(data);
                rle_decode(decoder, image);
            }
            FormatSliceFile::Svg(_file) => {
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

        fn rle_encode<Encoder: EncodableLayer>(
            info: SliceInfo,
            image: GrayImage,
            encoder: &mut Encoder,
        ) {
            let (width, height) = (info.resolution.x as usize, info.resolution.y as usize);
            let image = Image::from_raw(width, height, image.into_raw());

            for run in image.runs() {
                encoder.add_run(run.length, run.value)
            }
        }

        match self {
            FormatSliceFile::Goo(file) => {
                let mut encoder = goo_format::LayerEncoder::new();
                rle_encode(info, image, &mut encoder);

                let (data, checksum) = encoder.finish();
                let layer = &mut file.layers[layer];
                layer.data = data;
                layer.checksum = checksum;
            }
            FormatSliceFile::Ctb(file) => {
                let mut encoder = ctb_format::LayerEncoder::default();
                rle_encode(info, image, &mut encoder);
                file.layers[layer].data = encoder.into_inner();
            }
            FormatSliceFile::Svg(_file) => {}
        }
    }
}
