use std::fmt::{self, Debug};

use anyhow::{ensure, Result};

use common::serde::{Deserializer, Serializer, SizedString};

use crate::{PreviewImage, DELIMITER, MAGIC_TAG};

pub struct HeaderInfo {
    pub version: SizedString<4>,
    pub software_info: SizedString<32>,
    pub software_version: SizedString<24>,
    pub file_time: SizedString<24>,
    pub printer_name: SizedString<32>,
    pub printer_type: SizedString<32>,
    pub profile_name: SizedString<32>,
    pub anti_aliasing_level: u16,
    pub grey_level: u16,
    pub blur_level: u16,
    pub small_preview: PreviewImage<116, 116>,
    pub big_preview: PreviewImage<290, 290>,
    pub layer_count: u32,
    pub x_resolution: u16,
    pub y_resolution: u16,
    pub x_mirror: bool,
    pub y_mirror: bool,
    pub x_size: f32,
    pub y_size: f32,
    pub z_size: f32,
    pub layer_thickness: f32,
    pub exposure_time: f32,
    pub exposure_delay_mode: bool,
    pub turn_off_time: f32,
    pub bottom_before_lift_time: f32,
    pub bottom_after_lift_time: f32,
    pub bottom_after_retract_time: f32,
    pub before_lift_time: f32,
    pub after_lift_time: f32,
    pub after_retract_time: f32,
    pub bottom_exposure_time: f32,
    pub bottom_layers: u32,
    pub bottom_lift_distance: f32,
    pub bottom_lift_speed: f32,
    pub lift_distance: f32,
    pub lift_speed: f32,
    pub bottom_retract_distance: f32,
    pub bottom_retract_speed: f32,
    pub retract_distance: f32,
    pub retract_speed: f32,
    pub bottom_second_lift_distance: f32,
    pub bottom_second_lift_speed: f32,
    pub second_lift_distance: f32,
    pub second_lift_speed: f32,
    pub bottom_second_retract_distance: f32,
    pub bottom_second_retract_speed: f32,
    pub second_retract_distance: f32,
    pub second_retract_speed: f32,
    pub bottom_light_pwm: u16,
    pub light_pwm: u16,
    pub advance_mode: bool,
    pub printing_time: u32,
    pub total_volume: f32,
    pub total_weight: f32,
    pub total_price: f32,
    pub price_unit: SizedString<8>,
    pub grey_scale_level: bool,
    pub transition_layers: u16,
}

impl HeaderInfo {
    pub const SIZE: usize = 0x2FB95;
}

// this is fine
impl HeaderInfo {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_sized_string(&self.version);
        ser.write_bytes(MAGIC_TAG);
        ser.write_sized_string(&self.software_info);
        ser.write_sized_string(&self.software_version);
        ser.write_sized_string(&self.file_time);
        ser.write_sized_string(&self.printer_name);
        ser.write_sized_string(&self.printer_type);
        ser.write_sized_string(&self.profile_name);
        ser.write_u16(self.anti_aliasing_level);
        ser.write_u16(self.grey_level);
        ser.write_u16(self.blur_level);
        self.small_preview.serializes(ser);
        ser.write_bytes(DELIMITER);
        self.big_preview.serializes(ser);
        ser.write_bytes(DELIMITER);
        ser.write_u32(self.layer_count);
        ser.write_u16(self.x_resolution);
        ser.write_u16(self.y_resolution);
        ser.write_bool(self.x_mirror);
        ser.write_bool(self.y_mirror);
        ser.write_f32(self.x_size);
        ser.write_f32(self.y_size);
        ser.write_f32(self.z_size);
        ser.write_f32(self.layer_thickness);
        ser.write_f32(self.exposure_time);
        ser.write_bool(self.exposure_delay_mode);
        ser.write_f32(self.turn_off_time);
        ser.write_f32(self.bottom_before_lift_time);
        ser.write_f32(self.bottom_after_lift_time);
        ser.write_f32(self.bottom_after_retract_time);
        ser.write_f32(self.before_lift_time);
        ser.write_f32(self.after_lift_time);
        ser.write_f32(self.after_retract_time);
        ser.write_f32(self.bottom_exposure_time);
        ser.write_u32(self.bottom_layers);
        ser.write_f32(self.bottom_lift_distance);
        ser.write_f32(self.bottom_lift_speed);
        ser.write_f32(self.lift_distance);
        ser.write_f32(self.lift_speed);
        ser.write_f32(self.bottom_retract_distance);
        ser.write_f32(self.bottom_retract_speed);
        ser.write_f32(self.retract_distance);
        ser.write_f32(self.retract_speed);
        ser.write_f32(self.bottom_second_lift_distance);
        ser.write_f32(self.bottom_second_lift_speed);
        ser.write_f32(self.second_lift_distance);
        ser.write_f32(self.second_lift_speed);
        ser.write_f32(self.bottom_second_retract_distance);
        ser.write_f32(self.bottom_second_retract_speed);
        ser.write_f32(self.second_retract_distance);
        ser.write_f32(self.second_retract_speed);
        ser.write_u16(self.bottom_light_pwm);
        ser.write_u16(self.light_pwm);
        ser.write_bool(self.advance_mode);
        ser.write_u32(self.printing_time);
        ser.write_f32(self.total_volume);
        ser.write_f32(self.total_weight);
        ser.write_f32(self.total_price);
        ser.write_sized_string(&self.price_unit);
        ser.write_u32(Self::SIZE as u32);
        ser.write_bool(self.grey_scale_level);
        ser.write_u16(self.transition_layers);
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        let mut des = Deserializer::new(buf);

        let version = des.read_sized_string();
        ensure!(des.read_bytes(8) == [0x07, 0x00, 0x00, 0x00, 0x44, 0x4C, 0x50, 0x00]);
        let software_info = des.read_sized_string();
        let software_version = des.read_sized_string();
        let file_time = des.read_sized_string();
        let printer_name = des.read_sized_string();
        let printer_type = des.read_sized_string();
        let profile_name = des.read_sized_string();
        let anti_aliasing_level = des.read_u16();
        let grey_level = des.read_u16();
        let blur_level = des.read_u16();
        let small_preview = PreviewImage::deserializes(&mut des);
        ensure!(des.read_bytes(2) == [0xd, 0xa]);
        let big_preview = PreviewImage::deserializes(&mut des);
        ensure!(des.read_bytes(2) == [0xd, 0xa]);
        let layer_count = des.read_u32();
        let x_resolution = des.read_u16();
        let y_resolution = des.read_u16();
        let x_mirror = des.read_bool();
        let y_mirror = des.read_bool();
        let x_size = des.read_f32();
        let y_size = des.read_f32();
        let z_size = des.read_f32();
        let layer_thickness = des.read_f32();
        let exposure_time = des.read_f32();
        let exposure_delay_mode = des.read_bool();
        let turn_off_time = des.read_f32();
        let bottom_before_lift_time = des.read_f32();
        let bottom_after_lift_time = des.read_f32();
        let bottom_after_retract_time = des.read_f32();
        let before_lift_time = des.read_f32();
        let after_lift_time = des.read_f32();
        let after_retract_time = des.read_f32();
        let bottom_exposure_time = des.read_f32();
        let bottom_layers = des.read_u32();
        let bottom_lift_distance = des.read_f32();
        let bottom_lift_speed = des.read_f32();
        let lift_distance = des.read_f32();
        let lift_speed = des.read_f32();
        let bottom_retract_distance = des.read_f32();
        let bottom_retract_speed = des.read_f32();
        let retract_distance = des.read_f32();
        let retract_speed = des.read_f32();
        let bottom_second_lift_distance = des.read_f32();
        let bottom_second_lift_speed = des.read_f32();
        let second_lift_distance = des.read_f32();
        let second_lift_speed = des.read_f32();
        let bottom_second_retract_distance = des.read_f32();
        let bottom_second_retract_speed = des.read_f32();
        let second_retract_distance = des.read_f32();
        let second_retract_speed = des.read_f32();
        let bottom_light_pwm = des.read_u16();
        let light_pwm = des.read_u16();
        let advance_mode = des.read_bool();
        let printing_time = des.read_u32();
        let total_volume = des.read_f32();
        let total_weight = des.read_f32();
        let total_price = des.read_f32();
        let price_unit = des.read_sized_string();
        ensure!(des.read_u32() == Self::SIZE as u32);
        let grey_scale_level = des.read_bool();
        let transition_layers = des.read_u16();

        Ok(Self {
            version,
            software_info,
            software_version,
            file_time,
            printer_name,
            printer_type,
            profile_name,
            anti_aliasing_level,
            grey_level,
            blur_level,
            small_preview,
            big_preview,
            layer_count,
            x_resolution,
            y_resolution,
            x_mirror,
            y_mirror,
            x_size,
            y_size,
            z_size,
            layer_thickness,
            exposure_time,
            exposure_delay_mode,
            turn_off_time,
            bottom_before_lift_time,
            bottom_after_lift_time,
            bottom_after_retract_time,
            before_lift_time,
            after_lift_time,
            after_retract_time,
            bottom_exposure_time,
            bottom_layers,
            bottom_lift_distance,
            bottom_lift_speed,
            lift_distance,
            lift_speed,
            bottom_retract_distance,
            bottom_retract_speed,
            retract_distance,
            retract_speed,
            bottom_second_lift_distance,
            bottom_second_lift_speed,
            second_lift_distance,
            second_lift_speed,
            bottom_second_retract_distance,
            bottom_second_retract_speed,
            second_retract_distance,
            second_retract_speed,
            bottom_light_pwm,
            light_pwm,
            advance_mode,
            printing_time,
            total_volume,
            total_weight,
            total_price,
            price_unit,
            grey_scale_level,
            transition_layers,
        })
    }
}

impl Debug for HeaderInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeaderInfo")
            .field("version", &self.version)
            .field("software_info", &self.software_info)
            .field("software_version", &self.software_version)
            .field("file_time", &self.file_time)
            .field("printer_name", &self.printer_name)
            .field("printer_type", &self.printer_type)
            .field("profile_name", &self.profile_name)
            .field("anti_aliasing_level", &self.anti_aliasing_level)
            .field("grey_level", &self.grey_level)
            .field("blur_level", &self.blur_level)
            .field("layer_count", &self.layer_count)
            .field("x_resolution", &self.x_resolution)
            .field("y_resolution", &self.y_resolution)
            .field("x_mirror", &self.x_mirror)
            .field("y_mirror", &self.y_mirror)
            .field("x_size", &self.x_size)
            .field("y_size", &self.y_size)
            .field("z_size", &self.z_size)
            .field("layer_thickness", &self.layer_thickness)
            .field("exposure_time", &self.exposure_time)
            .field("exposure_delay_mode", &self.exposure_delay_mode)
            .field("turn_off_time", &self.turn_off_time)
            .field("bottom_before_lift_time", &self.bottom_before_lift_time)
            .field("bottom_after_lift_time", &self.bottom_after_lift_time)
            .field("bottom_after_retract_time", &self.bottom_after_retract_time)
            .field("before_lift_time", &self.before_lift_time)
            .field("after_lift_time", &self.after_lift_time)
            .field("after_retract_time", &self.after_retract_time)
            .field("bottom_exposure_time", &self.bottom_exposure_time)
            .field("bottom_layers", &self.bottom_layers)
            .field("bottom_lift_distance", &self.bottom_lift_distance)
            .field("bottom_lift_speed", &self.bottom_lift_speed)
            .field("lift_distance", &self.lift_distance)
            .field("lift_speed", &self.lift_speed)
            .field("bottom_retract_distance", &self.bottom_retract_distance)
            .field("bottom_retract_speed", &self.bottom_retract_speed)
            .field("retract_distance", &self.retract_distance)
            .field("retract_speed", &self.retract_speed)
            .field(
                "bottom_second_lift_distance",
                &self.bottom_second_lift_distance,
            )
            .field("bottom_second_lift_speed", &self.bottom_second_lift_speed)
            .field("second_lift_distance", &self.second_lift_distance)
            .field("second_lift_speed", &self.second_lift_speed)
            .field(
                "bottom_second_retract_distance",
                &self.bottom_second_retract_distance,
            )
            .field(
                "bottom_second_retract_speed",
                &self.bottom_second_retract_speed,
            )
            .field("second_retract_distance", &self.second_retract_distance)
            .field("second_retract_speed", &self.second_retract_speed)
            .field("bottom_light_pwm", &self.bottom_light_pwm)
            .field("light_pwm", &self.light_pwm)
            .field("advance_mode", &self.advance_mode)
            .field("printing_time", &self.printing_time)
            .field("total_volume", &self.total_volume)
            .field("total_weight", &self.total_weight)
            .field("total_price", &self.total_price)
            .field("price_unit", &self.price_unit)
            .field("grey_scale_level", &self.grey_scale_level)
            .field("transition_layers", &self.transition_layers)
            .finish()
    }
}
