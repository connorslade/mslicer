use anyhow::{Result, ensure};

use chrono::Local;
use common::{
    container::{Image, Run},
    progress::Progress,
    serde::{DynamicSerializer, Serializer, SizedString, SliceDeserializer},
    slice::{Format, SliceInfo, SliceResult, SlicedFile},
    units::Second,
};
use image::{RgbaImage, imageops::FilterType};
use nalgebra::{Vector2, Vector3};

use crate::{ENDING_STRING, Header, LayerContent, LayerDecoder, LayerEncoder, PreviewImage};

pub struct File {
    pub header: Header,
    pub layers: Vec<LayerContent>,
}

impl File {
    pub fn new(header: Header, layers: Vec<LayerContent>) -> Self {
        Self { header, layers }
    }

    pub fn from_slice_result(result: SliceResult<LayerContent>) -> Self {
        let slice_config = result.slice_config;
        let layers = result.layers.len() as u32;

        let print_time = slice_config.print_time(layers);
        let save_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self::new(
            Header {
                x_resolution: slice_config.platform_resolution.x as u16,
                y_resolution: slice_config.platform_resolution.y as u16,
                x_size: slice_config.platform_size.x,
                y_size: slice_config.platform_size.y,
                z_size: slice_config.platform_size.z,

                layer_count: layers,
                printing_time: print_time.get::<Second>() as u32,
                layer_thickness: slice_config.slice_height,
                bottom_layers: slice_config.first_layers,
                transition_layers: slice_config.transition_layers as u16,

                exposure_time: slice_config.exposure_config.exposure_time,
                lift_distance: slice_config.exposure_config.lift_distance,
                lift_speed: slice_config.exposure_config.lift_speed.convert(),
                retract_distance: slice_config.exposure_config.retract_distance,
                retract_speed: slice_config.exposure_config.retract_speed.convert(),

                bottom_exposure_time: slice_config.first_exposure_config.exposure_time,
                bottom_lift_distance: slice_config.first_exposure_config.lift_distance,
                bottom_lift_speed: slice_config.first_exposure_config.lift_speed.convert(),
                bottom_retract_distance: slice_config.first_exposure_config.retract_distance,
                bottom_retract_speed: slice_config.first_exposure_config.retract_speed.convert(),

                file_time: SizedString::new(save_time.as_bytes()),
                ..Default::default()
            },
            result.layers,
        )
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.header.serialize(ser);
        for layer in &self.layers {
            layer.serialize(ser);
        }
        ser.write_bytes(ENDING_STRING);
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        let header = Header::deserialize(des)?;
        let mut layers = Vec::with_capacity(header.layer_count as usize);

        for _ in 0..header.layer_count {
            layers.push(LayerContent::deserialize(des)?);
        }

        ensure!(des.read_slice(ENDING_STRING.len()) == ENDING_STRING);
        Ok(Self { header, layers })
    }
}

impl SlicedFile for File {
    fn serialize(&self, ser: &mut DynamicSerializer, progress: Progress) {
        self.serialize(ser);
        progress.set_total(1);
        progress.set_finished();
    }

    fn set_preview(&mut self, preview: &RgbaImage) {
        self.header.big_preview = PreviewImage::from_image_scaled(preview, FilterType::Nearest);
        self.header.small_preview = PreviewImage::from_image_scaled(preview, FilterType::Nearest);
    }

    fn info(&self) -> SliceInfo {
        SliceInfo {
            layers: self.layers.len() as u32,
            resolution: Vector2::new(
                self.header.x_resolution as u32,
                self.header.y_resolution as u32,
            ),
            size: Vector3::new(self.header.x_size, self.header.y_size, self.header.x_size),
            bottom_layers: self.header.bottom_layers,
        }
    }

    fn format(&self) -> Format {
        Format::Goo
    }

    fn runs(&self, layer: usize) -> Box<dyn Iterator<Item = Run> + '_> {
        let data = &self.layers[layer].data;
        Box::new(LayerDecoder::new(data))
    }

    fn overwrite_layer(&mut self, layer: usize, image: Image) {
        let mut encoder = LayerEncoder::new();
        (image.runs()).for_each(|run| encoder.add_run(run.length, run.value));
        let (data, checksum) = encoder.finish();

        let layer = &mut self.layers[layer];
        layer.data = data;
        layer.checksum = checksum;
    }
}
