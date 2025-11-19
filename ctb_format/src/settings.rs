use anyhow::Result;

use common::serde::Deserializer;
use nalgebra::{Vector2, Vector3};

use crate::{Section, crypto::decrypt, preview::PreviewImage};

#[derive(Debug)]
pub struct Settings {
    // Misc
    pub checksum_value: u64,
    pub layer_xor_key: u32,
    pub modified_timestamp_minutes: u32,
    pub disclaimer: Section,

    // Offsets
    pub layer_pointers_offset: u32,
    pub resin_parameters_address: u32,

    // Printer properties
    pub size: Vector3<f32>,
    pub resolution: Vector2<u32>,
    pub machine_name: String,
    pub projector_type: u32,

    // Operation properties
    pub total_height: f32,
    pub layer_height: f32,
    pub layer_count: u32,
    pub last_layer_index: u32,
    pub transition_layer_count: u32,
    // 7(0x7) [No AA] / 15(0x0F) [AA]
    pub anti_alias_flag: u8,
    pub anti_alias_level: u32,
    // 0 to not support, 0x40 to 0x50 to allow per layer parameters
    pub per_layer_settings: u8,

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

    // Derived values
    pub print_time: u32,
    pub material_milliliters: f32,
    pub material_grams: f32,
    pub material_cost: f32,
}

impl Settings {
    pub fn deserialize(main_des: &mut Deserializer, size: usize) -> Result<Self> {
        let bytes = decrypt(main_des.read_bytes(size));
        let mut des = Deserializer::new(&bytes);

        Ok(Self {
            checksum_value: des.read_u64_le(),
            layer_pointers_offset: des.read_u32_le(),
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
            layer_count: des.read_u32_le(),
            large_preview: {
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
            layer_xor_key: des.read_u32_le(),
            bottom_lift_height_2: des.read_f32_le(),
            bottom_lift_speed_2: des.read_f32_le(),
            lift_height_2: des.read_f32_le(),
            lift_speed_2: des.read_f32_le(),
            retract_height_2: des.read_f32_le(),
            retract_speed_2: des.read_f32_le(),
            rest_time_after_lift: des.read_f32_le(),
            machine_name: {
                let section = Section::deserialize_rev(&mut des)?;
                let machine_name = main_des.execute_at(section.offset as usize, |des| {
                    String::from_utf8_lossy(des.read_bytes(section.size as usize))
                });
                machine_name.trim_end_matches('\0').to_owned()
            },
            anti_alias_flag: des.read_u8(),
            per_layer_settings: {
                des.advance_by(2);
                des.read_u8()
            },
            modified_timestamp_minutes: des.read_u32_le(),
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
                Section::deserialize_rev(&mut des)?
            },
            resin_parameters_address: {
                des.advance_by(4);
                des.read_u32_le()
            },
        })
    }
}
