use std::fmt::Debug;

use anyhow::{Result, ensure};

use common::{
    serde::{Deserializer, Serializer, SizedString, SliceDeserializer},
    units::{Milimeters, MilimetersPerMinute, Seconds},
};

use crate::{DELIMITER, MAGIC_TAG, PreviewImage};

/// The header section of a `.goo` file.
#[derive(Debug)]
pub struct Header {
    /// Format version, should be "V3.0".
    pub version: SizedString<4>,
    /// Name of software that generated the file.
    pub software_info: SizedString<32>,
    /// Version of the slicer software.
    pub software_version: SizedString<24>,
    /// Time the file was created, recommended format is "%Y-%m-%d %H:%M:%S".
    pub file_time: SizedString<24>,
    /// Name of the printer the file was generated for.
    /// The default is "standard", but I don't think this field is used for anything.
    pub printer_name: SizedString<32>,
    /// Type of printer the file was generated for.
    /// The default is "Default", but I don't think this field is used for anything.
    pub printer_type: SizedString<32>,
    /// Name of the profile used to generate the file.
    /// I don't think this field is used for anything.
    pub profile_name: SizedString<32>,
    /// The anti-aliasing level used when generating the file.
    pub anti_aliasing_level: u16,
    /// Honestly not sure what this is.
    pub grey_level: u16,
    /// The blur level used when generating the file.
    pub blur_level: u16,
    /// 116 by 116 preview image.
    pub small_preview: PreviewImage<116, 116>,
    /// 290 by 290 preview image.
    pub big_preview: PreviewImage<290, 290>,
    /// Number of layers in the file.
    pub layer_count: u32,
    /// X resolution of the printer, in pixels.
    /// The sliced file will not print if the printer's resolution does not match this value.
    pub x_resolution: u16,
    /// Y resolution of the printer, in pixels.
    /// The sliced file will not print if the printer's resolution does not match this value.
    pub y_resolution: u16,
    /// Indicates if the print should be mirrored in the X direction.
    /// Not tested, so this might be wrong.
    pub x_mirror: bool,
    /// Indicates if the print should be mirrored in the Y direction.
    /// Not tested, so this might be wrong.
    pub y_mirror: bool,
    /// Size of the print area in the X direction.
    pub x_size: Milimeters,
    /// Size of the print area in the Y direction.
    pub y_size: Milimeters,
    /// Size of the print area in the Z direction.
    pub z_size: Milimeters,
    /// Thickness of each layer.
    pub layer_thickness: Milimeters,
    /// Default exposure time for each layer.
    pub exposure_time: Seconds,
    /// The exposure delay mode to use.
    /// ('turn off time' confuses me)
    pub exposure_delay_mode: ExposureDelayMode,
    /// Layer exposure delay when in [`ExposureDelayMode::TurnOffTime`].
    pub turn_off_time: Seconds,
    /// Time to wait before lifting the platform after exposing the bottom layers.
    /// When exposure delay mode is [`ExposureDelayMode::StaticTime`].
    pub bottom_before_lift_time: Seconds,
    /// Time to wait after lifting the platform after exposing the bottom layers.
    pub bottom_after_lift_time: Seconds,
    /// Time to wait after retracting the platform after exposing the bottom layers.
    pub bottom_after_retract_time: Seconds,
    /// Time to wait before lifting the platform after exposing each regular layer.
    pub before_lift_time: Seconds,
    /// Time to wait after lifting the platform after exposing each regular layer.
    pub after_lift_time: Seconds,
    /// Time to wait after retracting the platform after exposing each regular layer.
    pub after_retract_time: Seconds,
    /// Exposure time for the bottom layers.
    pub bottom_exposure_time: Seconds,
    /// Number of bottom layers.
    pub bottom_layers: u32,
    /// Distance to lift the platform after exposing each bottom layer.
    pub bottom_lift_distance: Milimeters,
    /// The speed to lift the platform after exposing each bottom layer.
    pub bottom_lift_speed: MilimetersPerMinute,
    /// Distance to lift the platform after exposing each regular layer.
    pub lift_distance: Milimeters,
    /// The speed to lift the platform after exposing each regular layer.
    pub lift_speed: MilimetersPerMinute,
    /// Distance to retract (move down) the platform after exposing each bottom layer.
    pub bottom_retract_distance: Milimeters,
    /// The speed to retract (move down) the platform after exposing each bottom layer.
    pub bottom_retract_speed: MilimetersPerMinute,
    /// Distance to retract (move down) the platform after exposing each regular layer.
    pub retract_distance: Milimeters,
    /// The speed to retract (move down) the platform after exposing each regular layer.
    pub retract_speed: MilimetersPerMinute,
    /// Second distance to lift the platform after exposing each bottom layer.
    pub bottom_second_lift_distance: Milimeters,
    /// The speed to lift the platform after exposing each bottom layer.
    pub bottom_second_lift_speed: MilimetersPerMinute,
    /// Second distance to lift the platform after exposing each regular layer.
    pub second_lift_distance: Milimeters,
    /// The speed to lift the platform after exposing each regular layer.
    pub second_lift_speed: MilimetersPerMinute,
    /// Second distance to retract (move down) the platform after exposing each bottom layer.
    pub bottom_second_retract_distance: Milimeters,
    /// The speed to retract (move down) the platform after exposing each bottom layer.
    pub bottom_second_retract_speed: MilimetersPerMinute,
    /// Second distance to retract (move down) the platform after exposing each regular layer.
    pub second_retract_distance: Milimeters,
    /// The speed to retract (move down) the platform after exposing each regular layer.
    pub second_retract_speed: MilimetersPerMinute,
    /// The power of the light for the bottom layers, 0-255.
    pub bottom_light_pwm: u8,
    /// The power of the light for the regular layers, 0-255.
    pub light_pwm: u8,
    /// If these global settings should be overwritten by each layers settings. Aka "Advanced Mode".
    pub per_layer_settings: bool,
    /// Estimated time to print the file, in seconds.
    pub printing_time: u32,
    /// Estimated volume of resin used, in mm^3.
    pub total_volume: f32,
    /// Estimated weight of resin used, in grams.
    pub total_weight: f32,
    /// Estimated price of resin used, in the currency specified by `price_unit`.
    pub total_price: f32,
    /// The currency symbol used for the price.
    pub price_unit: SizedString<8>,
    /// If false, layer gray values range from 0x00 to 0x0f, otherwise 0x00 to 0xff.
    pub grey_scale_level: bool,
    /// The number of layers to transition between bottom and regular exposure settings.
    pub transition_layers: u16,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ExposureDelayMode {
    TurnOffTime,
    StaticTime,
}

impl Header {
    pub const SIZE: usize = 0x2FB95;
}

// this is fine
impl Header {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.version.serialize(ser);
        ser.write_bytes(MAGIC_TAG);
        self.software_info.serialize(ser);
        self.software_version.serialize(ser);
        self.file_time.serialize(ser);
        self.printer_name.serialize(ser);
        self.printer_type.serialize(ser);
        self.profile_name.serialize(ser);
        ser.write_u16_be(self.anti_aliasing_level);
        ser.write_u16_be(self.grey_level);
        ser.write_u16_be(self.blur_level);
        self.small_preview.serializes(ser);
        ser.write_bytes(DELIMITER);
        self.big_preview.serializes(ser);
        ser.write_bytes(DELIMITER);
        ser.write_u32_be(self.layer_count);
        ser.write_u16_be(self.x_resolution);
        ser.write_u16_be(self.y_resolution);
        ser.write_bool(self.x_mirror);
        ser.write_bool(self.y_mirror);
        ser.write_f32_be(self.x_size.raw());
        ser.write_f32_be(self.y_size.raw());
        ser.write_f32_be(self.z_size.raw());
        ser.write_f32_be(self.layer_thickness.raw());
        ser.write_f32_be(self.exposure_time.raw());
        ser.write_u8(self.exposure_delay_mode as u8);
        ser.write_f32_be(self.turn_off_time.raw());
        ser.write_f32_be(self.bottom_before_lift_time.raw());
        ser.write_f32_be(self.bottom_after_lift_time.raw());
        ser.write_f32_be(self.bottom_after_retract_time.raw());
        ser.write_f32_be(self.before_lift_time.raw());
        ser.write_f32_be(self.after_lift_time.raw());
        ser.write_f32_be(self.after_retract_time.raw());
        ser.write_f32_be(self.bottom_exposure_time.raw());
        ser.write_u32_be(self.bottom_layers);
        ser.write_f32_be(self.bottom_lift_distance.raw());
        ser.write_f32_be(self.bottom_lift_speed.raw());
        ser.write_f32_be(self.lift_distance.raw());
        ser.write_f32_be(self.lift_speed.raw());
        ser.write_f32_be(self.bottom_retract_distance.raw());
        ser.write_f32_be(self.bottom_retract_speed.raw());
        ser.write_f32_be(self.retract_distance.raw());
        ser.write_f32_be(self.retract_speed.raw());
        ser.write_f32_be(self.bottom_second_lift_distance.raw());
        ser.write_f32_be(self.bottom_second_lift_speed.raw());
        ser.write_f32_be(self.second_lift_distance.raw());
        ser.write_f32_be(self.second_lift_speed.raw());
        ser.write_f32_be(self.bottom_second_retract_distance.raw());
        ser.write_f32_be(self.bottom_second_retract_speed.raw());
        ser.write_f32_be(self.second_retract_distance.raw());
        ser.write_f32_be(self.second_retract_speed.raw());
        ser.write_u16_be(self.bottom_light_pwm as u16);
        ser.write_u16_be(self.light_pwm as u16);
        ser.write_bool(self.per_layer_settings);
        ser.write_u32_be(self.printing_time);
        ser.write_f32_be(self.total_volume);
        ser.write_f32_be(self.total_weight);
        ser.write_f32_be(self.total_price);
        self.price_unit.serialize(ser);
        ser.write_u32_be(Self::SIZE as u32);
        ser.write_bool(self.grey_scale_level);
        ser.write_u16_be(self.transition_layers);
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            version: SizedString::deserialize(des),
            software_info: {
                ensure!(des.read_slice(8) == [0x07, 0x00, 0x00, 0x00, 0x44, 0x4C, 0x50, 0x00]);
                SizedString::deserialize(des)
            },
            software_version: SizedString::deserialize(des),
            file_time: SizedString::deserialize(des),
            printer_name: SizedString::deserialize(des),
            printer_type: SizedString::deserialize(des),
            profile_name: SizedString::deserialize(des),
            anti_aliasing_level: des.read_u16_be(),
            grey_level: des.read_u16_be(),
            blur_level: des.read_u16_be(),
            small_preview: PreviewImage::deserializes(des),
            big_preview: {
                ensure!(des.read_slice(2) == [0xd, 0xa]);
                PreviewImage::deserializes(des)
            },
            layer_count: {
                ensure!(des.read_slice(2) == [0xd, 0xa]);
                des.read_u32_be()
            },
            x_resolution: des.read_u16_be(),
            y_resolution: des.read_u16_be(),
            x_mirror: des.read_bool(),
            y_mirror: des.read_bool(),
            x_size: Milimeters::new(des.read_f32_be()),
            y_size: Milimeters::new(des.read_f32_be()),
            z_size: Milimeters::new(des.read_f32_be()),
            layer_thickness: Milimeters::new(des.read_f32_be()),
            exposure_time: Seconds::new(des.read_f32_be()),
            exposure_delay_mode: ExposureDelayMode::from_bool(des.read_bool()),
            turn_off_time: Seconds::new(des.read_f32_be()),
            bottom_before_lift_time: Seconds::new(des.read_f32_be()),
            bottom_after_lift_time: Seconds::new(des.read_f32_be()),
            bottom_after_retract_time: Seconds::new(des.read_f32_be()),
            before_lift_time: Seconds::new(des.read_f32_be()),
            after_lift_time: Seconds::new(des.read_f32_be()),
            after_retract_time: Seconds::new(des.read_f32_be()),
            bottom_exposure_time: Seconds::new(des.read_f32_be()),
            bottom_layers: des.read_u32_be(),
            bottom_lift_distance: Milimeters::new(des.read_f32_be()),
            bottom_lift_speed: MilimetersPerMinute::new(des.read_f32_be()),
            lift_distance: Milimeters::new(des.read_f32_be()),
            lift_speed: MilimetersPerMinute::new(des.read_f32_be()),
            bottom_retract_distance: Milimeters::new(des.read_f32_be()),
            bottom_retract_speed: MilimetersPerMinute::new(des.read_f32_be()),
            retract_distance: Milimeters::new(des.read_f32_be()),
            retract_speed: MilimetersPerMinute::new(des.read_f32_be()),
            bottom_second_lift_distance: Milimeters::new(des.read_f32_be()),
            bottom_second_lift_speed: MilimetersPerMinute::new(des.read_f32_be()),
            second_lift_distance: Milimeters::new(des.read_f32_be()),
            second_lift_speed: MilimetersPerMinute::new(des.read_f32_be()),
            bottom_second_retract_distance: Milimeters::new(des.read_f32_be()),
            bottom_second_retract_speed: MilimetersPerMinute::new(des.read_f32_be()),
            second_retract_distance: Milimeters::new(des.read_f32_be()),
            second_retract_speed: MilimetersPerMinute::new(des.read_f32_be()),
            bottom_light_pwm: des.read_u16_be().min(255) as u8,
            light_pwm: des.read_u16_be().min(255) as u8,
            per_layer_settings: des.read_bool(),
            printing_time: des.read_u32_be(),
            total_volume: des.read_f32_be(),
            total_weight: des.read_f32_be(),
            total_price: des.read_f32_be(),
            price_unit: SizedString::deserialize(des),
            grey_scale_level: {
                ensure!(des.read_u32_be() == Self::SIZE as u32);
                des.read_bool()
            },
            transition_layers: des.read_u16_be(),
        })
    }
}

impl ExposureDelayMode {
    pub fn from_bool(value: bool) -> Self {
        match value {
            false => ExposureDelayMode::TurnOffTime,
            true => ExposureDelayMode::StaticTime,
        }
    }
}
