use anyhow::Result;

use common::serde::{Deserializer, DynamicSerializer, Serializer};
use nalgebra::{Vector2, Vector3};

use crate::{
    Section,
    crypto::{decrypt, encrypt_in_place},
    preview::PreviewImage,
    resin::ResinParameters,
};

#[derive(Debug)]
pub struct Settings {
    // Misc
    pub checksum_value: u64,
    pub layer_xor_key: u32,
    pub disclaimer: Section,
    pub modified_timestamp_minutes: u32,
    pub layer_pointers_offset: u32,

    // Printer properties
    pub size: Vector3<f32>,
    pub resolution: Vector2<u32>,
    pub machine_name: String,
    pub projector_type: u32,
    pub resin_parameters: ResinParameters,

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
                let section = Section::deserialize(&mut des)?;
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
                Section::deserialize(&mut des)?
            },
            resin_parameters: {
                des.advance_by(4);
                let address = des.read_u32_le();
                main_des.execute_at(address as usize, |des| ResinParameters::deserialize(des))?
            },
        })
    }

    pub fn serialize<T: Serializer>(&self, main_ser: &mut T) -> usize {
        let mut ser = DynamicSerializer::new();
        ser.write_u64_le(self.checksum_value);
        ser.write_u32_le(self.layer_pointers_offset);
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
        ser.write_u32_le(self.layer_count);
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
        ser.write_u32_le(self.layer_xor_key);
        ser.write_f32_le(self.bottom_lift_height_2);
        ser.write_f32_le(self.bottom_lift_speed_2);
        ser.write_f32_le(self.lift_height_2);
        ser.write_f32_le(self.lift_speed_2);
        ser.write_f32_le(self.retract_height_2);
        ser.write_f32_le(self.retract_speed_2);
        ser.write_f32_le(self.rest_time_after_lift);
        let machine_name = ser.reserve(4);
        ser.write_u8(self.anti_alias_flag);
        ser.write_u16_le(0);
        ser.write_u8(self.per_layer_settings);
        ser.write_u32_le(self.modified_timestamp_minutes);
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
        Section::new(0, 0).serialize(&mut ser);
        ser.write_u32_le(0);
        let resin_parameters = ser.reserve(4);
        ser.reserve(4 * 2);

        let settings = main_ser.reserve(ser.pos());

        let machine_name_bytes = self.machine_name.as_bytes();
        let machine_name_pos = main_ser.pos();
        main_ser.write_bytes(machine_name_bytes);
        main_ser.execute_at(machine_name, |ser| {
            Section::new(machine_name_pos, machine_name_bytes.len()).serialize(ser);
        });

        main_ser.execute_at(resin_parameters, |ser| self.resin_parameters.serialize(ser));

        let pos = main_ser.pos();
        self.large_preview.serialize(main_ser);
        main_ser.execute_at(large_preview, |ser| ser.write_u32_le(pos as u32));

        let pos = main_ser.pos();
        self.small_preview.serialize(main_ser);
        main_ser.execute_at(small_preview, |ser| ser.write_u32_le(pos as u32));

        let mut settings_bytes = ser.into_inner();
        encrypt_in_place(&mut settings_bytes);
        main_ser.execute_at(settings, |ser| {
            ser.write_bytes(&settings_bytes);
        });

        settings_bytes.len()
    }
}
