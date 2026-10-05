use std::fmt::{self, Debug};

use anyhow::{Ok, Result};
use common::{
    serde::{Deserializer, SliceDeserializer},
    units::{Milimeters, MilimetersPerMinute, Milliliters, Seconds},
};
use nalgebra::{Vector2, Vector3};

use crate::{
    shared::{PreviewImage, Section, read_string},
    v4::Layer,
};

// todo: use consistent naming between v4 and v5 implementations

pub struct File {
    pub layers: Vec<Layer>,

    pub size: Vector3<Milimeters>,
    pub resolution: Vector2<u32>,

    pub large_preview: PreviewImage,
    pub small_preview: PreviewImage,

    pub total_height: Milimeters,
    pub print_time: u32, // what unit??

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

#[derive(Debug)]
pub struct PrintParameters {
    pub lift_height: Milimeters,
    pub lift_speed: MilimetersPerMinute,
    pub light_off_delay: Seconds,
    pub retract_speed: MilimetersPerMinute,

    pub bottom_lift_height: Milimeters,
    pub bottom_lift_speed: MilimetersPerMinute,
    pub bottom_light_off_delay: Seconds,

    pub volume: Milliliters,
    pub weight: f32, // grams (todo: make unit type?)
    pub cost: f32,
}

#[derive(Debug)]
pub struct SlicerParameters {
    pub bottom_lift_height_2: Milimeters,
    pub bottom_lift_speed_2: MilimetersPerMinute,
    pub lift_height_2: Milimeters,
    pub lift_speed_2: MilimetersPerMinute,
    pub retract_height_2: Milimeters,
    pub retract_speed_2: MilimetersPerMinute,
    pub rest_time_after_lift: Seconds,
    pub machine_name: String,
    pub anti_alias_flag: u8,
    pub per_layer_settings: u8,
    pub timestamp: u32,
    pub anti_alias_level: u8,
    pub software_version: u32,
    pub rest_time_after_retract: Seconds,
    pub rest_time_after_lift_2: Seconds,
    pub transition_layers: u32,
}

impl File {
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        assert_eq!(des.read_u32_le(), 0x12FD0106); // magic
        assert_eq!(des.read_u32_le(), 4); // version

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
            print_time: des.read_u32_le(),
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
}

impl PrintParameters {
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            bottom_lift_height: Milimeters::new(des.read_f32_le()),
            bottom_lift_speed: MilimetersPerMinute::new(des.read_f32_le()),
            lift_height: Milimeters::new(des.read_f32_le()),
            lift_speed: MilimetersPerMinute::new(des.read_f32_le()),
            retract_speed: MilimetersPerMinute::new(des.read_f32_le()),
            volume: Milliliters::new(des.read_f32_le()),
            weight: des.read_f32_le(),
            cost: des.read_f32_le(),
            bottom_light_off_delay: Seconds::new(des.read_f32_le()),
            light_off_delay: Seconds::new(des.read_f32_le()),
        })
    }
}

impl SlicerParameters {
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            bottom_lift_height_2: Milimeters::new(des.read_f32_le()),
            bottom_lift_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
            lift_height_2: Milimeters::new(des.read_f32_le()),
            lift_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
            retract_height_2: Milimeters::new(des.read_f32_le()),
            retract_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
            rest_time_after_lift: Seconds::new(des.read_f32_le()),
            machine_name: {
                let section = Section::deserialize(des)?;
                read_string(des, section).into_owned()
            },
            anti_alias_flag: des.read_u8(),
            per_layer_settings: des.read_u8(),
            timestamp: des.read_u32_be(),
            anti_alias_level: des.read_u8(),
            software_version: des.read_u32_le(),
            rest_time_after_retract: Seconds::new(des.read_f32_le()),
            rest_time_after_lift_2: Seconds::new(des.read_f32_le()),
            transition_layers: des.read_u32_le(),
        })
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
