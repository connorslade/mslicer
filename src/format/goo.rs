use super::serde::{Serializer, SizedString};

pub struct PreviewImage<const WIDTH: usize, const HEIGHT: usize> {
    data: [[u16; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> PreviewImage<WIDTH, HEIGHT> {
    pub const fn empty() -> Self {
        Self {
            data: [[0; WIDTH]; HEIGHT],
        }
    }

    pub fn serializes(&self, serializer: &mut Serializer) {
        for row in self.data.iter() {
            for pixel in row.iter() {
                serializer.write_u16(*pixel);
            }
        }
    }
}

pub struct EncodedLayer {
    data: Vec<u8>,
}

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

pub struct LayerContent {
    pub pause_flag: u16,
    pub pause_position_z: f32,
    pub layer_position_z: f32,
    pub layer_exposure_time: f32,
    pub layer_off_time: f32,
    pub before_lift_time: f32,
    pub after_lift_time: f32,
    pub after_retract_time: f32,
    pub lift_distance: f32,
    pub lift_speed: f32,
    pub second_lift_distance: f32,
    pub second_lift_speed: f32,
    pub retract_distance: f32,
    pub retract_speed: f32,
    pub second_retract_distance: f32,
    pub second_retract_speed: f32,
    pub light_pwm: u16,
    pub data: Vec<u8>,
}

impl HeaderInfo {
    pub const SIZE: usize = 0x2FB95;

    pub fn serialize(&self, buf: &mut [u8]) {
        let mut ser = Serializer::new(buf);
        ser.write_sized_string(&self.version);
        ser.write_bytes(&[0x07, 0x00, 0x00, 0x00, 0x44, 0x4C, 0x50, 0x00]);
        ser.write_sized_string(&self.software_info);
        ser.write_sized_string(&self.software_version);
        ser.write_sized_string(&self.file_time);
        ser.write_sized_string(&self.printer_name);
        ser.write_sized_string(&self.printer_type);
        ser.write_sized_string(&self.profile_name);
        ser.write_u16(self.anti_aliasing_level);
        ser.write_u16(self.grey_level);
        ser.write_u16(self.blur_level);
        self.small_preview.serializes(&mut ser);
        ser.write_bytes(&[0xd, 0xa]);
        self.big_preview.serializes(&mut ser);
        ser.write_bytes(&[0xd, 0xa]);
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
}

impl LayerContent {
    pub fn serialize(&self, buf: &mut [u8]) {
        let mut ser = Serializer::new(buf);
        ser.write_u16(self.pause_flag);
        ser.write_f32(self.pause_position_z);
        ser.write_f32(self.layer_position_z);
        ser.write_f32(self.layer_exposure_time);
        ser.write_f32(self.layer_off_time);
        ser.write_f32(self.before_lift_time);
        ser.write_f32(self.after_lift_time);
        ser.write_f32(self.after_retract_time);
        ser.write_f32(self.lift_distance);
        ser.write_f32(self.lift_speed);
        ser.write_f32(self.second_lift_distance);
        ser.write_f32(self.second_lift_speed);
        ser.write_f32(self.retract_distance);
        ser.write_f32(self.retract_speed);
        ser.write_f32(self.second_retract_distance);
        ser.write_f32(self.second_retract_speed);
        ser.write_u16(self.light_pwm);
        ser.write_u32(self.data.len() as u32 + 2);
        ser.write_bytes(&[0x55]);
        ser.write_bytes(&self.data);
        ser.write_u8(self.calculate_checksum());
        ser.write_bytes(&[0xd, 0xa]);
    }

    fn calculate_checksum(&self) -> u8 {
        let mut out = 0u8;
        for &byte in self.data.iter() {
            out = out.wrapping_add(byte);
        }
        !out
    }
}

impl Default for HeaderInfo {
    fn default() -> Self {
        Self {
            version: SizedString::new(b"V3.0"),
            software_info: SizedString::new(b"sla_slicer by Connor Slade"),
            software_version: SizedString::new(b"0.1.0"),
            file_time: SizedString::new(b"2024-06-14 08:10:14"),
            printer_name: SizedString::new(b"standard"),
            printer_type: SizedString::new(b"Default"),
            profile_name: SizedString::new(b"New Script"),
            anti_aliasing_level: 8,
            grey_level: 0,
            blur_level: 0,
            small_preview: PreviewImage::empty(),
            big_preview: PreviewImage::empty(),
            layer_count: 171,
            x_resolution: 11520,
            y_resolution: 5102,
            x_mirror: false,
            y_mirror: false,
            x_size: 218.88,
            y_size: 122.88,
            z_size: 260.0,
            layer_thickness: 0.05,
            exposure_time: 3.0,
            exposure_delay_mode: true,
            turn_off_time: 0.0,
            bottom_before_lift_time: 0.0,
            bottom_after_lift_time: 0.0,
            bottom_after_retract_time: 0.0,
            before_lift_time: 0.0,
            after_lift_time: 0.0,
            after_retract_time: 0.0,
            bottom_exposure_time: 50.0,
            bottom_layers: 8,
            bottom_lift_distance: 5.0,
            bottom_lift_speed: 65.0,
            lift_distance: 5.0,
            lift_speed: 65.0,
            bottom_retract_distance: 5.0,
            bottom_retract_speed: 150.0,
            retract_distance: 5.0,
            retract_speed: 0.0,
            bottom_second_lift_distance: 0.0,
            bottom_second_lift_speed: 0.0,
            second_lift_distance: 0.0,
            second_lift_speed: 0.0,
            bottom_second_retract_distance: 0.0,
            bottom_second_retract_speed: 0.0,
            second_retract_distance: 0.0,
            second_retract_speed: 0.0,
            bottom_light_pwm: 255,
            light_pwm: 255,
            advance_mode: false,
            printing_time: 2659,
            total_volume: 526.507,
            total_weight: 0.684,
            total_price: 0.0,
            price_unit: SizedString::new(b"$"),
            grey_scale_level: true,
            transition_layers: 10,
        }
    }
}

impl Default for LayerContent {
    fn default() -> Self {
        Self {
            pause_flag: 0,
            pause_position_z: 200.0,
            layer_position_z: 0.05,
            layer_exposure_time: 50.0,
            layer_off_time: 0.0,
            before_lift_time: 0.0,
            after_lift_time: 0.0,
            after_retract_time: 0.0,
            lift_distance: 5.0,
            lift_speed: 65.0,
            second_lift_distance: 0.0,
            second_lift_speed: 0.0,
            retract_distance: 5.0,
            retract_speed: 150.0,
            second_retract_distance: 0.0,
            second_retract_speed: 0.0,
            light_pwm: 255,
            data: Vec::new(),
        }
    }
}
