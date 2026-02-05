use std::{
    fmt::Debug,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Result, ensure};

use common::{
    misc::SliceResult,
    serde::{Deserializer, DynamicSerializer, Serializer, SliceDeserializer},
};
use nalgebra::{Vector2, Vector3, Vector4};
use sha2::{Digest, Sha256};

use crate::{
    Section,
    crypto::{decrypt, encrypt, encrypt_in_place},
    layer::{Layer, LayerRef},
    preview::PreviewImage,
    resin::ResinParameters,
};

const FORMAT_VERSION: u32 = 5;
const PAGE_SIZE: u64 = 1 << 32;
const DEFAULT_XOR_KEY: u32 = 0x67;
const DISCLAIMER: &str = "Layout and record format for the ctb and cbddlp file types are the copyrighted programs or codes of CBD Technology (China) Inc..The Customer or User shall not in any manner reproduce, distribute, modify, decompile, disassemble, decrypt, extract, reverse engineer, lease, assign, or sublicense the said programs or codes.";

pub struct File {
    pub layers: Vec<Layer>,

    // Misc
    pub checksum: u64,
    pub disclaimer: String,
    pub modified: u32, // Timestamp in minutes

    // Printer properties
    pub size: Vector3<f32>,
    pub resolution: Vector2<u32>,
    pub machine_name: String,
    pub projector_type: u32,
    pub resin_parameters: ResinParameters,

    // Operation properties
    pub total_height: f32,
    pub layer_height: f32,
    pub last_layer_index: u32,
    pub transition_layer_count: u32,
    pub anti_alias_flag: u8, // 7(0x7) [No AA] / 15(0x0F) [AA]
    pub anti_alias_level: u32,
    pub per_layer_settings: u8, // 0 to not support, 0x40 to 0x50 to allow per layer parameters

    // Derived values
    pub print_time: u32,
    pub material_milliliters: f32,
    pub material_grams: f32,
    pub material_cost: f32,

    // Preview Images
    pub large_preview: PreviewImage,
    pub small_preview: PreviewImage,

    // Layer config
    pub exposure_time: f32,
    pub bottom_exposure_time: f32,
    pub light_off_delay: f32,
    pub bottom_layer_count: u32,
    pub bottom_lift_height: f32,
    pub bottom_lift_speed: f32,
    pub lift_height: f32,
    pub lift_speed: f32,
    pub retract_speed: f32,
    pub bottom_light_off_delay: f32,
    pub light_pwm: u16,
    pub bottom_light_pwm: u16,
    pub bottom_lift_height_2: f32,
    pub bottom_lift_speed_2: f32,
    pub lift_height_2: f32,
    pub lift_speed_2: f32,
    pub retract_height_2: f32,
    pub retract_speed_2: f32,
    pub rest_time_after_lift: f32,
    pub bottom_retract_speed: f32,
    pub bottom_retract_speed_2: f32,
    pub rest_time_after_retract_2: f32,
    pub rest_time_after_lift_3: f32,
    pub rest_time_before_lift: f32,
    pub bottom_retract_height_2: f32,
    pub rest_time_after_retract: f32,
    pub rest_time_after_lift_2: f32,
}

impl File {
    pub fn deserialize(main_des: &mut SliceDeserializer) -> Result<Self> {
        assert_eq!(main_des.read_u32_le(), 0x12FD0107);
        let settings = Section::deserialize_rev(main_des)?;

        main_des.advance_by(4);
        let version = main_des.read_u32_le();
        ensure!(version == FORMAT_VERSION);
        let signature = Section::deserialize_rev(main_des)?;

        main_des.jump_to(settings.offset as usize);
        let bytes = decrypt(main_des.read_slice(settings.size as usize));
        let mut des = SliceDeserializer::new(&bytes);

        let checksum = des.read_u64_le();
        let hash = Sha256::digest(checksum.to_le_bytes());
        let signature = main_des.execute_at(signature.offset as usize, |des| {
            des.read_slice(signature.size as usize)
        });
        ensure!(&encrypt(&hash) == signature);

        let layer_offset = des.read_u32_le();
        let layer_count;

        Ok(Self {
            checksum,
            size: Vector3::new(des.read_f32_le(), des.read_f32_le(), des.read_f32_le()),
            total_height: {
                des.advance_by(4 * 2);
                des.read_f32_le()
            },
            layer_height: des.read_f32_le(),
            exposure_time: des.read_f32_le(),
            bottom_exposure_time: des.read_f32_le(),
            light_off_delay: des.read_f32_le(),
            bottom_layer_count: des.read_u32_le(),
            resolution: Vector2::new(des.read_u32_le(), des.read_u32_le()),
            large_preview: {
                layer_count = des.read_u32_le();
                main_des.jump_to(des.read_u32_le() as usize);
                PreviewImage::deserialize(main_des)?
            },
            small_preview: {
                main_des.jump_to(des.read_u32_le() as usize);
                PreviewImage::deserialize(main_des)?
            },
            print_time: des.read_u32_le(),
            projector_type: des.read_u32_le(),
            bottom_lift_height: des.read_f32_le(),
            bottom_lift_speed: des.read_f32_le(),
            lift_height: des.read_f32_le(),
            lift_speed: des.read_f32_le(),
            retract_speed: des.read_f32_le(),
            material_milliliters: des.read_f32_le(),
            material_grams: des.read_f32_le(),
            material_cost: des.read_f32_le(),
            bottom_light_off_delay: des.read_f32_le(),
            light_pwm: {
                des.advance_by(4);
                des.read_u16_le()
            },
            bottom_light_pwm: des.read_u16_le(),
            layers: {
                let xor_key = des.read_u32_le();
                let mut layers = Vec::with_capacity(layer_count as usize);

                main_des.jump_to(layer_offset as usize);

                for i in 0..layer_count {
                    let refrence = LayerRef::deserialize(main_des)?;
                    let position = refrence.page_number as usize * PAGE_SIZE as usize
                        + refrence.layer_offset as usize;

                    let layer =
                        main_des.execute_at(position, |des| Layer::deserialize(des, xor_key, i))?;
                    layers.push(layer);
                }

                layers
            },
            bottom_lift_height_2: des.read_f32_le(),
            bottom_lift_speed_2: des.read_f32_le(),
            lift_height_2: des.read_f32_le(),
            lift_speed_2: des.read_f32_le(),
            retract_height_2: des.read_f32_le(),
            retract_speed_2: des.read_f32_le(),
            rest_time_after_lift: des.read_f32_le(),
            machine_name: {
                let section = Section::deserialize(&mut des)?;
                let machine_name = main_des.execute_at(section.offset as usize, |des| {
                    String::from_utf8_lossy(des.read_slice(section.size as usize))
                });
                machine_name.trim_end_matches('\0').to_owned()
            },
            anti_alias_flag: des.read_u8(),
            per_layer_settings: {
                des.advance_by(2);
                des.read_u8()
            },
            modified: des.read_u32_le(),
            anti_alias_level: des.read_u32_le(),
            rest_time_after_retract: des.read_f32_le(),
            rest_time_after_lift_2: des.read_f32_le(),
            transition_layer_count: des.read_u32_le(),
            bottom_retract_speed: des.read_f32_le(),
            bottom_retract_speed_2: des.read_f32_le(),
            rest_time_after_retract_2: {
                des.advance_by(4 * 4);
                des.read_f32_le()
            },
            rest_time_after_lift_3: des.read_f32_le(),
            rest_time_before_lift: des.read_f32_le(),
            bottom_retract_height_2: des.read_f32_le(),
            last_layer_index: {
                des.advance_by(4 * 3);
                des.read_u32_le()
            },
            disclaimer: {
                des.advance_by(4 * 4);
                let section = Section::deserialize(&mut des)?;
                main_des.execute_at(section.offset as usize, |des| {
                    String::from_utf8_lossy(des.read_slice(section.size as usize)).into_owned()
                })
            },
            resin_parameters: {
                des.advance_by(4);
                let address = des.read_u32_le();
                main_des.execute_at(address as usize, |des| ResinParameters::deserialize(des))?
            },
        })
    }

    pub fn serialize<T: Serializer>(&self, main_ser: &mut T) {
        main_ser.write_u32_le(0x12FD0107);
        let settings_section = main_ser.reserve(8);
        main_ser.write_u32_le(0);
        main_ser.write_u32_le(FORMAT_VERSION);
        let signature = main_ser.reserve(8);
        main_ser.write_u32_le(0);
        main_ser.write_u16_le(1);
        main_ser.write_u16_le(1);
        main_ser.write_u32_le(0);
        main_ser.write_u32_le(0x2A);
        main_ser.write_u32_le(0);

        let pos = main_ser.pos();

        let mut ser = DynamicSerializer::new();
        ser.write_u64_le(self.checksum);
        let layer_offset = ser.reserve(4);
        ser.write_f32_le(self.size.x);
        ser.write_f32_le(self.size.y);
        ser.write_f32_le(self.size.z);
        ser.write_u32_le(0);
        ser.write_u32_le(0);
        ser.write_f32_le(self.total_height);
        ser.write_f32_le(self.layer_height);
        ser.write_f32_le(self.exposure_time);
        ser.write_f32_le(self.bottom_exposure_time);
        ser.write_f32_le(self.light_off_delay);
        ser.write_u32_le(self.bottom_layer_count);
        ser.write_u32_le(self.resolution.x);
        ser.write_u32_le(self.resolution.y);
        ser.write_u32_le(self.layers.len() as u32);
        let large_preview = ser.reserve(4);
        let small_preview = ser.reserve(4);
        ser.write_u32_le(self.print_time);
        ser.write_u32_le(self.projector_type);
        ser.write_f32_le(self.bottom_lift_height);
        ser.write_f32_le(self.bottom_lift_speed);
        ser.write_f32_le(self.lift_height);
        ser.write_f32_le(self.lift_speed);
        ser.write_f32_le(self.retract_speed);
        ser.write_f32_le(self.material_milliliters);
        ser.write_f32_le(self.material_grams);
        ser.write_f32_le(self.material_cost);
        ser.write_f32_le(self.bottom_light_off_delay);
        ser.write_u32_le(1);
        ser.write_u16_le(self.light_pwm);
        ser.write_u16_le(self.bottom_light_pwm);
        ser.write_u32_le(DEFAULT_XOR_KEY);
        ser.write_f32_le(self.bottom_lift_height_2);
        ser.write_f32_le(self.bottom_lift_speed_2);
        ser.write_f32_le(self.lift_height_2);
        ser.write_f32_le(self.lift_speed_2);
        ser.write_f32_le(self.retract_height_2);
        ser.write_f32_le(self.retract_speed_2);
        ser.write_f32_le(self.rest_time_after_lift);
        let machine_name = ser.reserve(8);
        ser.write_u8(self.anti_alias_flag);
        ser.write_u16_le(0);
        ser.write_u8(self.per_layer_settings);
        ser.write_u32_le(self.modified);
        ser.write_u32_le(self.anti_alias_level);
        ser.write_f32_le(self.rest_time_after_retract);
        ser.write_f32_le(self.rest_time_after_lift_2);
        ser.write_u32_le(self.transition_layer_count);
        ser.write_f32_le(self.bottom_retract_speed);
        ser.write_f32_le(self.bottom_retract_speed_2);
        ser.write_u32_le(0);
        ser.write_f32_le(4.0);
        ser.write_u32_le(0);
        ser.write_f32_le(4.0);
        ser.write_f32_le(self.rest_time_after_retract_2);
        ser.write_f32_le(self.rest_time_after_lift_3);
        ser.write_f32_le(self.rest_time_before_lift);
        ser.write_f32_le(self.bottom_retract_height_2);
        ser.reserve(4 * 2);
        ser.write_u32_le(4);
        ser.write_u32_le(self.last_layer_index);
        ser.reserve(4 * 4);
        let disclaimer = ser.reserve(8);
        ser.write_u32_le(0);
        let resin = ser.reserve(4);
        ser.reserve(4 * 2);

        let settings = main_ser.reserve(ser.pos().next_multiple_of(32));

        let mut machine_name_bytes = self.machine_name.as_bytes().to_vec();
        machine_name_bytes.push(0);

        let machine_name_pos = main_ser.pos();
        main_ser.write_bytes(&machine_name_bytes);
        ser.execute_at(machine_name, |ser| {
            Section::new(machine_name_pos, machine_name_bytes.len()).serialize(ser);
        });

        let disclaimer_bytes = self.disclaimer.as_bytes();
        ser.execute_at(disclaimer, |ser| {
            Section::new(main_ser.pos(), disclaimer_bytes.len()).serialize(ser);
        });
        main_ser.write_bytes(disclaimer_bytes);

        ser.execute_at(resin, |ser| ser.write_u32_le(main_ser.pos() as u32));
        self.resin_parameters.serialize(main_ser);

        ser.execute_at(large_preview, |ser| ser.write_u32_le(main_ser.pos() as u32));
        self.large_preview.serialize(main_ser);

        ser.execute_at(small_preview, |ser| ser.write_u32_le(main_ser.pos() as u32));
        self.small_preview.serialize(main_ser);

        ser.execute_at(layer_offset, |ser| ser.write_u32_le(main_ser.pos() as u32));
        let layer_refs = main_ser.reserve(16 * self.layers.len());

        let mut settings_bytes = ser.into_inner();
        encrypt_in_place(&mut settings_bytes);
        main_ser.execute_at(settings, |ser| {
            ser.write_bytes(&settings_bytes);
        });

        main_ser.execute_at(settings_section, |ser| {
            Section::new(pos, settings_bytes.len()).serialize_rev(ser)
        });

        let hash = Sha256::digest(self.checksum.to_le_bytes());
        let bytes = encrypt(&hash);

        main_ser.write_u32_le(0x422052FA);
        main_ser.write_u32_le(0);

        let pos = main_ser.pos();
        main_ser.write_bytes(&bytes);
        main_ser.execute_at(signature, |ser| {
            Section::new(pos, bytes.len()).serialize_rev(ser);
        });

        main_ser.write_u32_le(0x6D4232B3);

        for (i, layer) in self.layers.iter().enumerate() {
            let cursor = main_ser.pos() as u64;

            let page_number = (cursor / PAGE_SIZE) as u32;
            let refrence = LayerRef {
                layer_offset: (cursor % PAGE_SIZE) as u32,
                page_number,
            };
            main_ser.execute_at(layer_refs + i * 16, |ser| refrence.serialize(ser));

            layer.serialize(main_ser, DEFAULT_XOR_KEY, page_number, i as u32);
        }
    }
}

impl File {
    pub fn from_slice_result(result: SliceResult<Layer>) -> Self {
        let config = result.slice_config;
        let layer_count = result.layers.len();

        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            layers: result.layers,
            checksum: 0,
            disclaimer: DISCLAIMER.into(),
            modified: (epoch / 60) as u32,
            size: config.platform_size,
            resolution: config.platform_resolution,
            machine_name: "Unknown".into(),
            projector_type: 1,
            resin_parameters: ResinParameters {
                resin_color: Vector4::zeros(),
                machine_name: "Unknown".into(),
                resin_type: "Normal".into(),
                resin_name: "Standard".into(),
                resin_density: 1.1,
            },
            total_height: layer_count as f32 * config.slice_height,
            layer_height: config.slice_height,
            last_layer_index: layer_count.saturating_sub(1) as u32,
            transition_layer_count: config.transition_layers,
            anti_alias_flag: 7,
            anti_alias_level: 0,
            per_layer_settings: 0,
            print_time: 0,
            material_milliliters: 0.0,
            material_grams: 0.0,
            material_cost: 0.0,
            large_preview: PreviewImage::default(),
            small_preview: PreviewImage::default(),
            exposure_time: config.exposure_config.exposure_time,
            bottom_exposure_time: config.first_exposure_config.exposure_time,
            light_off_delay: 0.0,
            bottom_layer_count: config.first_layers,
            bottom_lift_height: config.first_exposure_config.lift_distance,
            bottom_lift_speed: config.first_exposure_config.lift_speed * 600.0,
            lift_height: config.exposure_config.lift_distance,
            lift_speed: config.exposure_config.lift_speed * 600.0,
            retract_speed: config.exposure_config.retract_speed * 600.0,
            bottom_light_off_delay: 0.0,
            light_pwm: 255,
            bottom_light_pwm: 255,
            bottom_lift_height_2: 4.0,  // make a setting
            bottom_lift_speed_2: 320.0, // make a setting
            lift_height_2: 0.0,
            lift_speed_2: 0.0,
            retract_height_2: 0.0,
            retract_speed_2: 0.0,
            rest_time_after_lift: 0.0,
            bottom_retract_speed: config.first_exposure_config.retract_speed * 60.0,
            bottom_retract_speed_2: 90.0, // make a setting
            rest_time_after_retract_2: 1.0,
            rest_time_after_lift_3: 0.0,
            rest_time_before_lift: 0.0,
            bottom_retract_height_2: 1.5,
            rest_time_after_retract: 1.0,
            rest_time_after_lift_2: 0.0,
        }
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("checksum", &self.checksum)
            .field("disclaimer", &self.disclaimer)
            .field("modified", &self.modified)
            .field("size", &self.size)
            .field("resolution", &self.resolution)
            .field("machine_name", &self.machine_name)
            .field("projector_type", &self.projector_type)
            .field("resin_parameters", &self.resin_parameters)
            .field("total_height", &self.total_height)
            .field("layer_height", &self.layer_height)
            .field("last_layer_index", &self.last_layer_index)
            .field("transition_layer_count", &self.transition_layer_count)
            .field("anti_alias_flag", &self.anti_alias_flag)
            .field("anti_alias_level", &self.anti_alias_level)
            .field("per_layer_settings", &self.per_layer_settings)
            .field("print_time", &self.print_time)
            .field("material_milliliters", &self.material_milliliters)
            .field("material_grams", &self.material_grams)
            .field("material_cost", &self.material_cost)
            .field("large_preview", &self.large_preview)
            .field("small_preview", &self.small_preview)
            .field("exposure_time", &self.exposure_time)
            .field("bottom_exposure_time", &self.bottom_exposure_time)
            .field("light_off_delay", &self.light_off_delay)
            .field("bottom_layer_count", &self.bottom_layer_count)
            .field("bottom_lift_height", &self.bottom_lift_height)
            .field("bottom_lift_speed", &self.bottom_lift_speed)
            .field("lift_height", &self.lift_height)
            .field("lift_speed", &self.lift_speed)
            .field("retract_speed", &self.retract_speed)
            .field("bottom_light_off_delay", &self.bottom_light_off_delay)
            .field("light_pwm", &self.light_pwm)
            .field("bottom_light_pwm", &self.bottom_light_pwm)
            .field("bottom_lift_height_2", &self.bottom_lift_height_2)
            .field("bottom_lift_speed_2", &self.bottom_lift_speed_2)
            .field("lift_height_2", &self.lift_height_2)
            .field("lift_speed_2", &self.lift_speed_2)
            .field("retract_height_2", &self.retract_height_2)
            .field("retract_speed_2", &self.retract_speed_2)
            .field("rest_time_after_lift", &self.rest_time_after_lift)
            .field("bottom_retract_speed", &self.bottom_retract_speed)
            .field("bottom_retract_speed_2", &self.bottom_retract_speed_2)
            .field("rest_time_after_retract_2", &self.rest_time_after_retract_2)
            .field("rest_time_after_lift_3", &self.rest_time_after_lift_3)
            .field("rest_time_before_lift", &self.rest_time_before_lift)
            .field("bottom_retract_height_2", &self.bottom_retract_height_2)
            .field("rest_time_after_retract", &self.rest_time_after_retract)
            .field("rest_time_after_lift_2", &self.rest_time_after_lift_2)
            .finish()
    }
}
