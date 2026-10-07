use std::{
    fmt::{self, Debug},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Ok, Result, ensure};
use common::{
    progress::Progress,
    serde::{Deserializer, DynamicSerializer, Serializer, SliceDeserializer},
    slice::{self, ExposureConfig, Height, SliceConfig, SliceMode, SlicedFile},
    units::{Milimeter, Milimeters, MilimetersPerMinute, Milliliters, Second, Seconds},
};
use image::RgbaImage;
use nalgebra::{Vector2, Vector3};

use crate::{
    shared::{
        DEFAULT_XOR_KEY, DISCLAIMER, PAGE_SIZE, PreviewImage, Section, scale_preview, xor_cypher,
    },
    v4::{
        Layer,
        layer::{LAYER_DEF_EXT_SIZE, LAYER_DEF_SIZE},
        params::{PrintParameters, PrinterParametersExt, SlicerParameters},
    },
};

const MAGIC: u32 = 0x12FD0106;
const VERSION: u32 = 4;

// todo: use consistent naming between v4 and v5 implementations

pub struct File {
    pub layers: Vec<Layer>,

    pub size: Vector3<Milimeters>,
    pub resolution: Vector2<u32>,

    pub large_preview: PreviewImage,
    pub small_preview: PreviewImage,

    pub total_height: Milimeters,
    pub print_time: Seconds,

    pub layer_height: Milimeters,
    pub exposure_time: Seconds,
    pub bottom_exposure_time: Seconds,
    pub light_off_delay: Seconds,
    pub bottom_layer_count: u32,
    pub projector_type: u32,
    pub anti_alias: u32,
    pub light_pwm: u16,
    pub bottom_light_pwm: u16,

    pub print_parameters: PrintParameters,
    pub slicer_parameters: SlicerParameters,
}

impl File {
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        ensure!(des.read_u32_le() == MAGIC);
        ensure!(des.read_u32_le() == VERSION);

        let layers; // (offset, count)
        Ok(Self {
            size: Vector3::new(des.read_f32_le(), des.read_f32_le(), des.read_f32_le())
                .map(Milimeters::new),
            total_height: {
                des.advance_by(8); // no idea
                Milimeters::new(des.read_f32_le())
            },
            layer_height: Milimeters::new(des.read_f32_le()),
            exposure_time: Seconds::new(des.read_f32_le()),
            bottom_exposure_time: Seconds::new(des.read_f32_le()),
            light_off_delay: Seconds::new(des.read_f32_le()),
            bottom_layer_count: des.read_u32_le(),
            resolution: Vector2::new(des.read_u32_le(), des.read_u32_le()),
            large_preview: {
                let offset = des.read_u32_le() as usize;
                des.execute_at(offset, |des| PreviewImage::deserialize(des))?
            },
            small_preview: {
                layers = Section::deserialize(des)?;
                let offset = des.read_u32_le() as usize;
                des.execute_at(offset, |des| PreviewImage::deserialize(des))?
            },
            print_time: Seconds::new(des.read_u32_le() as f32),
            projector_type: des.read_u32_le(),
            print_parameters: {
                let section = Section::deserialize(des)?;
                des.execute_at(section.offset as usize, |des| {
                    PrintParameters::deserialize(des)
                })?
            },
            anti_alias: des.read_u32_le(),
            light_pwm: des.read_u16_le(),
            bottom_light_pwm: des.read_u16_le(),
            layers: {
                let key = des.read_u32_le();
                let start = des.pos();
                des.jump_to(layers.offset as usize);

                let mut out = Vec::new();
                for i in 0..layers.size {
                    out.push(Layer::deserialize(des, key, i)?);
                }

                des.jump_to(start);
                out
            },
            slicer_parameters: {
                let section = Section::deserialize(des)?;
                des.execute_at(section.offset as usize, |des| {
                    SlicerParameters::deserialize(des)
                })?
            },
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(MAGIC);
        ser.write_u32_le(VERSION);

        ser.write_f32_le(self.size.x.get::<Milimeter>());
        ser.write_f32_le(self.size.y.get::<Milimeter>());
        ser.write_f32_le(self.size.z.get::<Milimeter>());
        ser.reserve(8);
        ser.write_f32_le(self.total_height.get::<Milimeter>());
        ser.write_f32_le(self.layer_height.get::<Milimeter>());
        ser.write_f32_le(self.exposure_time.get::<Second>());
        ser.write_f32_le(self.bottom_exposure_time.get::<Second>());
        ser.write_f32_le(self.light_off_delay.get::<Second>());
        ser.write_u32_le(self.bottom_layer_count);
        ser.write_u32_le(self.resolution.x);
        ser.write_u32_le(self.resolution.y);
        let large_preview = ser.reserve(4);
        let layers = ser.reserve(8);
        let small_preview = ser.reserve(4);
        ser.write_u32_le(self.print_time.get::<Second>().ceil() as u32);
        ser.write_u32_le(self.projector_type);
        let print_parameters = ser.reserve(8);
        ser.write_u32_le(self.anti_alias);
        ser.write_u16_le(self.light_pwm);
        ser.write_u16_le(self.bottom_light_pwm);
        ser.write_u32_le(DEFAULT_XOR_KEY); // (because layer data needs to be encrypted??)
        let slice_parameters = ser.reserve(8);

        let offset = ser.pos() as u32;
        ser.execute_at(large_preview, |ser| ser.write_u32_le(offset));
        self.large_preview.serialize(ser);

        let offset = ser.pos() as u32;
        ser.execute_at(small_preview, |ser| ser.write_u32_le(offset));
        self.small_preview.serialize(ser);

        let offset = ser.pos();
        self.print_parameters.serialize(ser);
        let section = Section::new(offset, ser.pos() - offset);
        ser.execute_at(print_parameters, |ser| section.serialize(ser));

        let section = Section::new(ser.pos(), 76);
        self.slicer_parameters.serialize(ser);
        ser.execute_at(slice_parameters, |ser| section.serialize(ser));

        let section = Section::new(ser.pos(), self.layers.len());
        ser.execute_at(layers, |ser| section.serialize(ser));
        let layer_refs = ser.reserve(LAYER_DEF_SIZE * self.layers.len());

        for (i, layer) in self.layers.iter().enumerate() {
            let data = (ser.pos() + LAYER_DEF_EXT_SIZE) as u64;
            let (page, offset) = ((data / PAGE_SIZE) as u32, (data % PAGE_SIZE) as u32);

            layer.serialize_ext(ser, page, offset);

            let data = ser.pos();
            ser.write_bytes(&layer.data);
            let buffer = ser.view_mut(data, layer.data.len());
            xor_cypher(buffer, DEFAULT_XOR_KEY, i as u32);

            ser.execute_at(layer_refs + LAYER_DEF_SIZE * i, |ser| {
                layer.serialize_ref(ser, page, offset)
            });
        }
    }
}

impl File {
    pub fn from_layers(config: &SliceConfig, layers: Vec<Layer>) -> Self {
        let (bottom_layer_count, transition_layers) = config.layer_counts();
        let last_layer_idx = layers.len().saturating_sub(1) as u32;
        let total_height = (layers.last()).map(|x| x.position_z).unwrap_or_default();

        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            layers,
            size: config.platform_size,
            resolution: config.platform_resolution,
            large_preview: PreviewImage::default(),
            small_preview: PreviewImage::default(),
            total_height,
            print_time: Seconds::new(0.0),
            layer_height: config.slice_height,
            exposure_time: config.exposure_config.exposure_time,
            bottom_exposure_time: config.first_exposure_config.exposure_time,
            light_off_delay: Seconds::new(0.0),
            bottom_layer_count,
            projector_type: 1,
            anti_alias: 1,
            light_pwm: config.exposure_config.pwm as u16,
            bottom_light_pwm: config.first_exposure_config.pwm as u16,
            print_parameters: PrintParameters {
                lift_height: config.exposure_config.lift_distance,
                lift_speed: config.exposure_config.lift_speed.convert(),
                light_off_delay: Seconds::new(0.0),
                retract_speed: config.exposure_config.retract_speed.convert(),
                bottom_lift_height: config.first_exposure_config.lift_distance,
                bottom_lift_speed: config.first_exposure_config.lift_speed.convert(),
                bottom_light_off_delay: Seconds::new(0.0),
                volume: Milliliters::new(0.0),
                weight: 0.0,
                cost: 0.0,
                bottom_layer_count,
            },
            slicer_parameters: SlicerParameters {
                bottom_lift_height_2: Milimeters::new(0.0), // make a setting
                bottom_lift_speed_2: MilimetersPerMinute::new(320.0), // make a setting
                lift_height_2: Milimeters::new(0.0),
                lift_speed_2: MilimetersPerMinute::new(0.0),
                retract_height_2: Milimeters::new(0.0),
                retract_speed_2: MilimetersPerMinute::new(0.0),
                rest_time_after_lift: Seconds::new(0.0),
                machine_name: "Unknown".into(),
                anti_alias_flag: 7,
                per_layer_settings: 0x40,
                timestamp: (epoch / 60) as u32,
                anti_alias_level: 1,
                software_version: 0x01090000,
                rest_time_after_retract: config.exposure_config.exposure_delay,
                rest_time_after_lift_2: Seconds::new(0.0),
                transition_layers,
                ext: PrinterParametersExt {
                    bottom_retract_speed: config.first_exposure_config.retract_speed.convert(),
                    bottom_retract_speed_2: MilimetersPerMinute::new(90.0), // make a setting
                    rest_time_after_retract: config.exposure_config.exposure_delay,
                    rest_time_after_lift: Seconds::new(0.0),
                    rest_time_before_lift: Seconds::new(0.0),
                    bottom_retract_height_2: Milimeters::new(0.0),
                    last_layer_idx,
                    disclaimer: DISCLAIMER.into(),
                },
            },
        }
    }
}

impl SlicedFile for File {
    fn serialize(&self, ser: &mut DynamicSerializer, progress: &Progress) {
        progress.set_total(1);
        self.serialize(ser);
        progress.set_finished();
    }

    fn set_preview(&mut self, preview: &RgbaImage) {
        let (large, small) = scale_preview(preview);
        self.large_preview = large;
        self.small_preview = small;
    }

    fn slice_config(&self) -> SliceConfig {
        SliceConfig {
            mode: SliceMode::Raster,
            supersample: Default::default(),
            exposure_remap: Default::default(),
            platform_resolution: self.resolution,
            platform_size: self.size,
            slice_height: self.layer_height,
            exposure_config: ExposureConfig {
                exposure_time: self.exposure_time,
                exposure_delay: self.slicer_parameters.ext.rest_time_after_retract,
                pwm: self.light_pwm as u8,
                lift_distance: self.print_parameters.lift_height
                    - self.slicer_parameters.lift_height_2,
                lift_speed: self.print_parameters.lift_speed.convert(),
                retract_speed: self.print_parameters.retract_speed.convert(),
            },
            first_exposure_config: ExposureConfig {
                exposure_time: self.bottom_exposure_time,
                exposure_delay: self.slicer_parameters.ext.rest_time_after_retract, // idk
                pwm: self.bottom_light_pwm as u8,
                lift_distance: self.print_parameters.bottom_lift_height
                    - self.slicer_parameters.bottom_lift_height_2,
                lift_speed: self.print_parameters.bottom_lift_speed.convert(),
                retract_speed: self.slicer_parameters.ext.bottom_retract_speed.convert(),
            },
            first_layers: Height::Layers(self.bottom_layer_count),
            transition_layers: Height::Layers(self.slicer_parameters.transition_layers),
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
        [&self.large_preview, &self.small_preview]
            .map(|x| x.into_image())
            .to_vec()
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("layers", &self.layers.len())
            .field("size", &self.size.as_slice())
            .field("resolution", &self.resolution.as_slice())
            .field("large_preview", &self.large_preview)
            .field("small_preview", &self.small_preview)
            .field("total_height", &self.total_height)
            .field("print_time", &self.print_time)
            .field("layer_height", &self.layer_height)
            .field("exposure_time", &self.exposure_time)
            .field("bottom_exposure_time", &self.bottom_exposure_time)
            .field("light_off_delay", &self.light_off_delay)
            .field("bottom_layer_count", &self.bottom_layer_count)
            .field("projector_type", &self.projector_type)
            .field("anti_alias", &self.anti_alias)
            .field("light_pwm", &self.light_pwm)
            .field("bottom_light_pwm", &self.bottom_light_pwm)
            .field("print_parameters", &self.print_parameters)
            .field("slicer_parameters", &self.slicer_parameters)
            .finish()
    }
}
