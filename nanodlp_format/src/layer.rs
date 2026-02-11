use common::{
    container::{
        Clusters, Image, ImageRuns,
        rle::{
            self, Run,
            png::{ColorType, PngEncoder},
        },
    },
    serde::DynamicSerializer,
    slice::{EncodableLayer, SliceConfig},
    units::Milimeter,
};
use image::{GrayImage, RgbImage};
use nalgebra::Vector2;

use crate::{decode_png, types::LayerInfo};

pub struct Layer {
    pub inner: Vec<u8>,
    pub info: LayerInfo,
}

pub struct LayerEncoder {
    platform: Vector2<u32>,
    runs: Vec<Run>,
}

pub struct LayerDecoder {
    image: RgbImage,
}

impl LayerEncoder {
    pub fn from_gray_image(gray_image: GrayImage) -> Self {
        let platform = Vector2::new(gray_image.width(), gray_image.height());
        let image = Image::from_raw(
            gray_image.width() as usize,
            gray_image.height() as usize,
            gray_image.into_raw(),
        );

        let mut out = LayerEncoder::new(platform);
        (image.runs()).for_each(|run| out.add_run(run.length, run.value));
        out
    }

    pub fn image_data(self) -> Vec<u8> {
        let mut ser = DynamicSerializer::new();

        let resolution = Vector2::new(self.platform.x.div_ceil(3), self.platform.y);
        let mut encoder = PngEncoder::new(&mut ser, ColorType::Truecolor, resolution);
        encoder.write_pixel_dimensions(3, 1);
        encoder.write_image_data(self.runs);
        encoder.write_end();
        ser.into_inner()
    }
}

impl EncodableLayer for LayerEncoder {
    type Output = Layer;

    fn new(platform: Vector2<u32>) -> Self {
        Self {
            platform,
            runs: Vec::new(),
        }
    }

    fn add_run(&mut self, length: u64, value: u8) {
        self.runs.push(Run { length, value });
    }

    fn finish(self, _layer: u64, config: &SliceConfig) -> Self::Output {
        let nonzero = rle::bits::from_runs(&self.runs);
        let chunks = rle::bits::chunks(&nonzero, config.platform_resolution.x as u64);

        let mut min = Vector2::repeat(u64::MAX);
        let mut max = Vector2::repeat(u64::MIN);
        let mut islands = Clusters::default();

        let width = config.platform_resolution.x as u64;
        for row in 1..chunks.len() {
            row_bounds(&chunks[row], width, row, &mut min, &mut max);
            rle::bits::cluster_row_adjacency(&mut islands, &chunks, row - 1, row);
        }

        let islands = (islands.clusters())
            .map(|(_, runs)| runs.iter().map(|(_, _, s)| s).sum::<u64>())
            .collect::<Vec<_>>();
        let smallest_area = islands.iter().min().copied().unwrap_or_default();
        let largest_area = islands.iter().max().copied().unwrap_or_default();
        let total_area = islands.iter().sum::<u64>();

        let pixel_area = config.platform_size.x.get::<Milimeter>()
            * config.platform_size.y.get::<Milimeter>()
            / config.platform_resolution.x as f32
            / config.platform_resolution.y as f32;

        Layer {
            info: LayerInfo {
                total_solid_area: total_area as f32 * pixel_area,
                largest_area: largest_area as f32 * pixel_area,
                smallest_area: smallest_area as f32 * pixel_area,
                min_x: min.x as u32,
                min_y: min.y as u32,
                max_x: max.x as u32,
                max_y: max.y as u32,
                area_count: islands.len() as u32,
            },
            inner: self.image_data(),
        }
    }
}

impl LayerDecoder {
    pub fn new(data: &[u8]) -> Self {
        Self {
            image: decode_png(data).unwrap().to_rgb8(),
        }
    }

    pub fn runs(&self) -> impl Iterator<Item = Run> {
        ImageRuns::new(self.image.as_raw())
    }

    pub fn into_inner(self) -> RgbImage {
        self.image
    }
}

fn row_bounds(row: &[u64], width: u64, y: usize, min: &mut Vector2<u64>, max: &mut Vector2<u64>) {
    if row.len() <= 1 {
        return;
    }

    // If len>1 then there is at least one non-zero voxel in that layer, which
    // will extend the bounding box's y component.
    min.y = min.y.min(y as u64);
    max.y = max.y.max(y as u64);

    // The left side of the first run starts at row[0] pixels in and the right
    // side ends row[-1] pixels in from the right (when the last run is
    // nonzero).
    let offset = (row.len() % 2 == 1)
        .then(|| *row.last().unwrap())
        .unwrap_or_default();
    min.x = min.x.min(row[0]);
    max.x = max.x.max(width - offset);
}
