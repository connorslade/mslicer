use anyhow::{Result, ensure};

use chrono::Local;
use common::{
    progress::Progress,
    serde::{DynamicSerializer, Serializer, SizedString, SliceDeserializer},
    slice::{self, ExposureConfig, ExposureRemap, SliceConfig, SliceMode, SlicedFile},
    units::Second,
};
use image::{RgbaImage, imageops::FilterType};
use nalgebra::{Vector2, Vector3};

use crate::{ENDING_STRING, Header, Layer, PreviewImage};

/// A Goo file.
pub struct File {
    pub header: Header,
    pub layers: Vec<Layer>,
}

impl File {
    pub fn new(header: Header, layers: Vec<Layer>) -> Self {
        Self { header, layers }
    }

    pub fn from_layers(config: &SliceConfig, layers: Vec<Layer>) -> Self {
        let layer_count = layers.len() as u32;

        let print_time = config.print_time(layer_count);
        let save_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self::new(
            Header {
                x_resolution: config.platform_resolution.x as u16,
                y_resolution: config.platform_resolution.y as u16,
                x_size: config.platform_size.x,
                y_size: config.platform_size.y,
                z_size: config.platform_size.z,

                layer_count,
                printing_time: print_time.get::<Second>() as u32,
                layer_thickness: config.slice_height,
                bottom_layers: config.first_layers,
                transition_layers: config.transition_layers as u16,

                exposure_time: config.exposure_config.exposure_time,
                after_retract_time: config.exposure_config.exposure_delay,
                light_pwm: config.exposure_config.pwm,
                lift_distance: config.exposure_config.lift_distance,
                lift_speed: config.exposure_config.lift_speed.convert(),
                retract_distance: config.exposure_config.lift_distance,
                retract_speed: config.exposure_config.retract_speed.convert(),

                bottom_exposure_time: config.first_exposure_config.exposure_time,
                bottom_after_retract_time: config.first_exposure_config.exposure_delay,
                bottom_light_pwm: config.first_exposure_config.pwm,
                bottom_lift_distance: config.first_exposure_config.lift_distance,
                bottom_lift_speed: config.first_exposure_config.lift_speed.convert(),
                bottom_retract_distance: config.first_exposure_config.lift_distance,
                bottom_retract_speed: config.first_exposure_config.retract_speed.convert(),

                file_time: SizedString::new(save_time.as_bytes()),
                ..Default::default()
            },
            layers,
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
            layers.push(Layer::deserialize(des)?);
        }

        ensure!(des.read_slice(ENDING_STRING.len()) == ENDING_STRING);
        Ok(Self { header, layers })
    }
}

impl SlicedFile for File {
    fn serialize(&self, ser: &mut DynamicSerializer, progress: &Progress) {
        self.serialize(ser);
        progress.set_total(1);
        progress.set_finished();
    }

    fn set_preview(&mut self, preview: &RgbaImage) {
        self.header.big_preview = PreviewImage::from_image_scaled(preview, FilterType::Nearest);
        self.header.small_preview = PreviewImage::from_image_scaled(preview, FilterType::Nearest);
    }

    fn slice_config(&self) -> SliceConfig {
        let header = &self.header;
        SliceConfig {
            mode: SliceMode::Raster,
            supersample: Default::default(),
            exposure_remap: ExposureRemap::default(),
            platform_resolution: Vector2::new(header.x_resolution, header.y_resolution).cast(),
            platform_size: Vector3::new(header.x_size, header.y_size, header.x_size),
            slice_height: header.layer_thickness,
            exposure_config: ExposureConfig {
                exposure_time: header.exposure_time,
                exposure_delay: header.after_retract_time,
                pwm: header.light_pwm,
                lift_distance: header.lift_distance,
                lift_speed: header.lift_speed.convert(),
                retract_speed: header.retract_speed.convert(),
            },
            first_exposure_config: ExposureConfig {
                exposure_time: header.bottom_exposure_time,
                exposure_delay: header.bottom_after_retract_time,
                pwm: header.bottom_light_pwm,
                lift_distance: header.bottom_lift_distance,
                lift_speed: header.bottom_lift_speed.convert(),
                retract_speed: header.bottom_retract_speed.convert(),
            },
            first_layers: header.bottom_layers,
            transition_layers: header.transition_layers as u32,
        }
    }

    fn layer_count(&self) -> usize {
        self.layers.len()
    }

    fn layers(&self, progress: &Progress) -> Vec<slice::Layer> {
        progress.set_total(self.layers.len() as u64);
        (self.layers.iter())
            .map(|l| l.into_layer())
            .inspect(|_| progress.add_complete(1))
            .collect()
    }

    fn previews(&self) -> Vec<RgbaImage> {
        vec![
            self.header.big_preview.into_image(),
            self.header.small_preview.into_image(),
        ]
    }
}
