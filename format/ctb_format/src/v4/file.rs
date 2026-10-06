use std::fmt::{self, Debug};

use anyhow::{Ok, Result, ensure};
use common::{
    serde::{Deserializer, Serializer, SliceDeserializer},
    units::{
        Milimeter, Milimeters, MilimetersPerMinute, Milliliters, Minute, Minutes, Second, Seconds,
    },
};
use nalgebra::{Vector2, Vector3};

use crate::{
    shared::{PAGE_SIZE, PreviewImage, Section, read_string},
    v4::{
        Layer,
        layer::{LAYER_DEF_EXT_SIZE, LAYER_DEF_SIZE},
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
    pub print_time: Minutes,

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
    pub bottom_layer_count: u32, // duplicated 4 sum reason?? (so close to writing a chitu rant)
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
    pub anti_alias_flag: u8, // 0x07 no AA, 0x0F AA
    pub per_layer_settings: u8,
    pub timestamp: u32,
    pub anti_alias_level: u32,
    pub software_version: u32,
    pub rest_time_after_retract: Seconds,
    pub rest_time_after_lift_2: Seconds,
    pub transition_layers: u32,
    pub ext: PrinterParametersExt,
}

#[derive(Debug)]
pub struct PrinterParametersExt {
    pub bottom_retract_speed: MilimetersPerMinute,
    pub bottom_retract_speed_2: MilimetersPerMinute,
    pub rest_time_after_retract: Seconds,
    pub rest_time_after_lift: Seconds,
    pub rest_time_before_lift: Seconds,
    pub bottom_retract_height_2: Milimeters,
    pub last_layer_idx: u32,
    pub disclaimer: String,
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
            print_time: Minutes::new(des.read_u32_le() as f32),
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
        ser.write_u32_le(self.print_time.get::<Minute>().ceil() as u32);
        ser.write_u32_le(self.projector_type);
        let print_parameters = ser.reserve(8);
        ser.write_u32_le(self.anti_alias);
        ser.write_u16_le(self.light_pwm);
        ser.write_u16_le(self.bottom_light_pwm);
        ser.write_u32_le(0); // cypher key (because layer data needs to be encrypted??)
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

        let offset = ser.pos();
        self.slicer_parameters.serialize(ser);
        let section = Section::new(offset, ser.pos() - offset);
        ser.execute_at(slice_parameters, |ser| section.serialize(ser));

        let section = Section::new(ser.pos(), self.layers.len());
        ser.execute_at(layers, |ser| section.serialize(ser));
        let layer_refs = ser.reserve(LAYER_DEF_SIZE * self.layers.len());

        for (i, layer) in self.layers.iter().enumerate() {
            let data = (ser.pos() + LAYER_DEF_EXT_SIZE) as u64;
            let (page, offset) = ((data / PAGE_SIZE) as u32, (data % PAGE_SIZE) as u32);

            layer.serialize_ext(ser, page, offset);
            ser.write_bytes(&layer.data);
            ser.execute_at(layer_refs + LAYER_DEF_SIZE * i, |ser| {
                layer.serialize_ref(ser, page, offset)
            });
        }
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
            bottom_layer_count: des.read_u32_le(),
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_le(self.bottom_lift_height.get::<Milimeter>());
        ser.write_f32_le(self.bottom_lift_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.lift_height.get::<Milimeter>());
        ser.write_f32_le(self.lift_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.retract_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.volume.get::<Milimeter>());
        ser.write_f32_le(self.weight);
        ser.write_f32_le(self.cost);
        ser.write_f32_le(self.bottom_light_off_delay.get::<Second>());
        ser.write_f32_le(self.light_off_delay.get::<Second>());
        ser.write_u32_le(self.bottom_layer_count);
        ser.reserve(4 * 4);
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
            per_layer_settings: {
                des.advance_by(2);
                des.read_u8()
            },
            timestamp: des.read_u32_le(),
            anti_alias_level: des.read_u32_le(),
            software_version: des.read_u32_le(),
            rest_time_after_retract: Seconds::new(des.read_f32_le()),
            rest_time_after_lift_2: Seconds::new(des.read_f32_le()),
            transition_layers: des.read_u32_le(),
            ext: {
                let ext = des.read_u32_le() as usize;
                des.execute_at(ext, |des| PrinterParametersExt::deserialize(des))?
            },
            // 8 bytes of padding
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_le(self.bottom_lift_height_2.get::<Milimeter>());
        ser.write_f32_le(self.bottom_lift_speed_2.get::<Milimeter, Minute>());
        ser.write_f32_le(self.lift_height_2.get::<Milimeter>());
        ser.write_f32_le(self.lift_speed_2.get::<Milimeter, Minute>());
        ser.write_f32_le(self.retract_height_2.get::<Milimeter>());
        ser.write_f32_le(self.retract_speed_2.get::<Milimeter, Minute>());
        ser.write_f32_le(self.rest_time_after_lift.get::<Second>());
        let machine_name = ser.reserve(8);
        ser.write_u8(self.anti_alias_flag);
        ser.reserve(2);
        ser.write_u8(self.per_layer_settings);
        ser.write_u32_le(self.timestamp);
        ser.write_u32_le(self.anti_alias_level);
        ser.write_u32_le(self.software_version);
        ser.write_f32_le(self.rest_time_after_retract.get::<Second>());
        ser.write_f32_le(self.rest_time_after_lift_2.get::<Second>());
        ser.write_u32_le(self.transition_layers);
        let ext_offset = ser.reserve(4);
        ser.reserve(4 * 2);

        let section = Section::new(ser.pos(), self.machine_name.len());
        ser.execute_at(machine_name, |ser| section.serialize(ser));
        ser.write_bytes(self.machine_name.as_bytes());

        let offset = ser.pos() as u32;
        ser.execute_at(ext_offset, |ser| ser.write_u32_le(offset));
        self.ext.serialize(ser);
    }
}

impl PrinterParametersExt {
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        {
            Ok(Self {
                bottom_retract_speed: MilimetersPerMinute::new(des.read_f32_le()),
                bottom_retract_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
                rest_time_after_retract: {
                    des.advance_by(4 * 4);
                    Seconds::new(des.read_f32_le())
                },
                rest_time_after_lift: Seconds::new(des.read_f32_le()),
                rest_time_before_lift: Seconds::new(des.read_f32_le()),
                bottom_retract_height_2: Milimeters::new(des.read_f32_le()),
                last_layer_idx: {
                    des.advance_by(4 * 3);
                    des.read_u32_le()
                },
                disclaimer: {
                    des.advance_by(4 * 4);
                    let section = Section::deserialize(des)?;
                    read_string(des, section).into_owned()
                },
            })
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_le(self.bottom_retract_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.bottom_retract_speed_2.get::<Milimeter, Minute>());
        ser.reserve(4);
        ser.write_f32_le(4.0);
        ser.reserve(4);
        ser.write_f32_le(4.0);
        ser.write_f32_le(self.rest_time_after_retract.get::<Second>());
        ser.write_f32_le(self.rest_time_after_lift.get::<Second>());
        ser.write_f32_le(self.rest_time_before_lift.get::<Second>());
        ser.write_f32_le(self.bottom_retract_height_2.get::<Milimeter>());
        ser.write_f32_le(2955.996);
        ser.reserve(4);
        ser.write_u32_le(5);
        ser.write_u32_le(self.last_layer_idx);
        ser.reserve(4 * 4);
        let disclaimer = ser.reserve(8);
        ser.reserve(384 + 4);

        let section = Section::new(ser.pos(), self.disclaimer.len());
        ser.execute_at(disclaimer, |ser| section.serialize(ser));
        ser.write_bytes(self.disclaimer.as_bytes());
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
