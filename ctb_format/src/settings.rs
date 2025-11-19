use anyhow::Result;

use common::serde::Deserializer;

use crate::Section;

#[derive(Debug)]
pub struct Settings {
    pub checksum_value: u64,
    pub layer_pointers_offset: u32,
    pub x_size: f32,
    pub y_size: f32,
    pub z_size: f32,
    pub total_height: f32,
    pub layer_height: f32,
    pub exposure_time: f32,
    pub bottom_exposure_time: f32,
    pub light_off_delay: f32,
    pub bottom_layer_count: u32,
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub layer_count: u32,
    pub large_preview_offset: u32,
    pub small_preview_offset: u32,
    pub print_time: u32,
    pub projector_type: u32,
    pub bottom_lift_height: f32,
    pub bottom_lift_speed: f32,
    pub lift_height: f32,
    pub lift_speed: f32,
    pub retract_speed: f32,
    pub material_milliliters: f32,
    pub material_grams: f32,
    pub material_cost: f32,
    pub bottom_light_off_delay: f32,
    pub light_pwm: u16,
    pub bottom_light_pwm: u16,
    pub layer_xor_key: u32,
    pub bottom_lift_height_2: f32,
    pub bottom_lift_speed_2: f32,
    pub lift_height_2: f32,
    pub lift_speed_2: f32,
    pub retract_height_2: f32,
    pub retract_speed_2: f32,
    pub rest_time_after_lift: f32,
    pub machine_name: Section,
}

impl Settings {
    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        Ok(Self {
            checksum_value: des.read_u64_le(),
            layer_pointers_offset: des.read_u32_le(),
            x_size: des.read_f32_le(),
            y_size: des.read_f32_le(),
            z_size: des.read_f32_le(),
            total_height: {
                des.advance_by(4 * 2);
                des.read_f32_le()
            },
            layer_height: des.read_f32_le(),
            exposure_time: des.read_f32_le(),
            bottom_exposure_time: des.read_f32_le(),
            light_off_delay: des.read_f32_le(),
            bottom_layer_count: des.read_u32_le(),
            resolution_x: des.read_u32_le(),
            resolution_y: des.read_u32_le(),
            layer_count: des.read_u32_le(),
            large_preview_offset: des.read_u32_le(),
            small_preview_offset: des.read_u32_le(),
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
            machine_name: Section::deserialize_rev(des)?,
        })
    }
}
