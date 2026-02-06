use std::env;

use common::units::Micrometers;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct Meta {
    pub format_version: u32,
    pub distro: String,
    pub program: String,
    pub version: String,
    #[serde(rename = "OS")]
    pub os: String,
    pub arch: String,
    pub profile: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LayerInfo {
    pub total_solid_area: f32,
    pub largest_area: f32,
    pub smallest_area: f32,
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
    pub area_count: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct Plate {
    #[serde(rename = "PlateID")]
    pub plate_id: u32,
    #[serde(rename = "ProfileID")]
    // profile: null
    pub profile_id: u32,
    pub created_date: u32, // ← time
    pub stop_layers: String,
    pub path: String,
    pub low_quality_layer_number: u32,
    pub auto_center: u32,
    pub updated: u32, // ← time
    pub last_print: u32,
    pub print_time: u32,
    pub print_est: u32,
    pub image_rotate: u32,
    pub mask_effect: u32,
    pub x_res: u32,
    pub y_res: u32,
    pub z_res: u32,
    pub multi_cure: String,
    pub multi_thickness: String,
    // cure_times: null
    // dynamic_thickness: null
    pub offset: u32,
    // over_hangs: null
    pub risky: bool,
    pub is_faulty: bool,
    pub repaired: bool,
    pub corrupted: bool,
    // faulty_layers: null
    pub total_solid_area: f32,
    pub blackout_data: String,
    pub layers_count: u32,
    pub processed: bool,
    pub feedback: bool,
    pub re_slice_needed: bool,
    pub multi_material: bool,
    #[serde(rename = "PrintID")]
    pub print_id: u32,
    #[serde(rename = "MC")]
    pub mc: PlateMc,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub z_min: f32,
    pub z_max: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct PlateMc {
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    // x: null
    // y: null
    multi_cure_gap: u32,
    count: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct Options {
    pub _type: String,
    #[serde(rename = "URL")]
    pub url: String,
    pub p_width: u32,
    pub p_height: u32,
    pub scale_factor: u32,
    pub start_layer: u32,
    pub support_depth: u32,
    pub support_layer_number: u32,
    pub thickness: Micrometers,
    pub x_offset: u32,
    pub y_offset: u32,
    pub z_offset: u32,
    pub x_pixel_size: f32,
    pub y_pixel_size: f32,
    // mask: null,
    pub auto_center: u32,
    pub slice_from_zero: bool,
    pub disable_validator: bool,
    pub preview_generate: bool,
    pub running: bool,
    pub debug: bool,
    pub is_faulty: bool,
    pub corrupted: bool,
    pub multi_material: bool,
    pub adapt_export: String,
    pub preview_color: String,
    // faulty_layers: null
    // overhang_layers: null
    // layer_status: null
    pub boundary: Boundary,
    // area: ...
    #[serde(rename = "MC")]
    pub mc: PlateMc,
    pub multi_thickness: String,
    pub export_path: String,
    pub network_save: String,
    pub file: String,
    pub file_size: u32,
    pub adapt_slicing: u32,
    pub adapt_slicing_min: u32,
    pub adapt_slicing_max: u32,
    pub support_offset: u32,
    pub offset: u32,
    pub fill_color: String,
    pub blank_color: String,
    pub dim_amount: u32,
    pub dim_wall: u32,
    pub dim_skip: u32,
    pub pixel_diming: u32,
    pub hatching_type: u32,
    pub elephant_mid_exposure: u32,
    pub elephant_type: u32,
    pub elephant_amount: u32,
    pub elephant_wall: u32,
    pub elephant_layers: u32,
    pub hatching_wall: u32,
    pub hatching_gap: u32,
    pub hatching_outer_wall: u32,
    pub hatching_top_cap: u32,
    pub hatching_bottom_cap: u32,
    pub multi_cure_gap: u32,
    pub anti_alias: u32,
    pub anti_alias_3_d: u32,
    pub anti_alias_3_d_distance: u32,
    pub anti_alias_3_d_min: u32,
    pub anti_alias_threshold: u32,
    pub image_rotate: u32,
    pub ignore_mask: u32,
    #[serde(rename = "XYRes")]
    pub xy_res: f32,
    pub x_res: f32,
    pub y_res: f32,
    pub z_res_perc: u32,
    pub preview_width: u32,
    pub preview_height: u32,
    pub barrel_factor: u32,
    pub barrel_x: u32,
    pub barrel_y: u32,
    pub image_mirror: u32,
    pub display_controller: u32,
    pub light_output_formula: String,
    #[serde(rename = "PlateID")]
    pub plate_id: u32,
    #[serde(rename = "LayerID")]
    pub layer_id: u32,
    pub layer_count: u32,
    #[serde(rename = "UUID")]
    pub uuid: String,
    // dynamic_thickness: null
    #[serde(rename = "FillColorRGB")]
    pub fill_color_rgb: Color,
    #[serde(rename = "BlankColorRGB")]
    pub blank_color_rgb: Color,
    pub export_type: u32,
    pub output_path: String,
    pub suffix: String,
    pub skip_empty: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct Boundary {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    z_min: f32,
    z_max: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct Color {
    r: u32,
    g: u32,
    b: u32,
    a: u32,
}

impl Color {
    pub fn repeat(val: u32) -> Self {
        Self {
            r: val,
            g: val,
            b: val,
            a: val,
        }
    }
}

pub const SHIELD_BEFORE_LAYER: &str = r#"G90\r\nSET_PROGRESS_BAR CL=[[LayerNumber]] TL=[[TotalNumberOfLayers]]\r\nMOVE_PLATE Z=[[LayerPosition]] F=600\r\n[[MoveWait 2]]\r\n[[PositionSet [[LayerPosition]]]]\r\n[[DynamicWaitStart]]\r\nfss_idle"#;
pub const SHIELD_AFTER_LAYER: &str = r#"[JS]if ([[LayerNumber]]==1||[[LayerNumber]]%3==0) output=\"TRIGGER\";[/JS]\r\n[JS]if ([[LayerNumber]]==1||[[LayerNumber]]%5==0) output=\"M105\";[/JS]\r\n[[MoveCounterSet 0]]\r\nUVLED_ON PWM=[[_UvPwmValue]]\r\nDWELL P={[[CureTime]]*1000}\r\n[[MoveWait 1]]\r\nUVLED_OFF \r\n[[GPIOHigh 10]]\r\n[[SeparationDetectionStart]]\r\nG91\r\n[[MoveCounterSet 0]]\r\nMOVE_PLATE_FSS Z=[[ZLiftDistance]] F={120-([[LayerNumber]]\u003c5)*70}\r\n[[MoveWait 1]]\r\n\r\n[[SeparationDetectionStop]]\r\n\r\n[[PositionChange [[ZLiftDistance]]]]\r\n[JS]if ([[LayerNumber]]==1||[[LayerNumber]]%100==0) output=\"G4 P2000\";[/JS]\r\n[JS]if ([[LayerNumber]]==1||[[LayerNumber]]%100==0) output=\"[[PressureWrite 1]]\";[/JS]"#;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct Profile {
    #[serde(rename = "ResinID")]
    pub resin_id: u32,
    #[serde(rename = "ProfileID")]
    pub profile_id: u32,
    pub title: String,
    pub desc: String,
    pub color: String,
    pub resin_price: u32,
    pub optimum_temperature: u32,
    pub depth: Micrometers,
    pub support_top_wait: u32,
    pub support_wait_height: u32,
    pub support_depth: Micrometers,
    pub support_wait_before_print: u32,
    pub support_wait_after_print: u32,
    pub transitional_layer: u32,
    pub updated: u32,
    pub custom_values: ProfileCustomValues,
    pub _type: u32,
    pub z_step_wait: u32,
    pub top_wait: u32,
    pub wait_height: u32,
    pub cure_time: f32,
    pub wait_before_print: u32,
    pub wait_after_print: u32,
    pub support_cure_time: f32,
    pub support_layer_number: u32,
    pub adapt_slicing: u32,
    pub adapt_slicing_min: u32,
    pub adapt_slicing_max: u32,
    pub support_offset: u32,
    pub offset: u32,
    pub fill_color: String,
    pub blank_color: String,
    pub dim_amount: u32,
    pub dim_wall: u32,
    pub dim_skip: u32,
    pub pixel_diming: u32,
    pub hatching_type: u32,
    pub elephant_mid_exposure: u32,
    pub elephant_type: u32,
    pub elephant_amount: u32,
    pub elephant_wall: u32,
    pub elephant_layers: u32,
    pub hatching_wall: u32,
    pub hatching_gap: u32,
    pub hatching_outer_wall: u32,
    pub hatching_top_cap: u32,
    pub hatching_bottom_cap: u32,
    pub multi_cure_gap: u32,
    pub anti_alias: u32,
    pub anti_alias_3_d: u32,
    pub anti_alias_3_d_distance: u32,
    pub anti_alias_3_d_min: u32,
    pub anti_alias_threshold: u32,
    pub image_rotate: u32,
    pub ignore_mask: u32,
    #[serde(rename = "XYRes")]
    pub xy_res: u32,
    pub y_res: u32,
    pub z_res_perc: u32,
    pub dynamic_cure_time: String,
    pub dynamic_speed: String,
    pub shield_before_layer: String,
    pub shield_after_layer: String,
    pub shield_during_cure: String,
    pub shield_start: String,
    pub shield_resume: String,
    pub shield_finish: String,
    pub laser_code: String,
    pub shutter_open_gcode: String,
    pub shutter_close_gcode: String,
    pub separation_detection: String,
    pub resin_level_detection: String,
    pub crash_detection: String,
    pub dynamic_wait: String,
    pub slow_section_height: u32,
    pub slow_section_step_wait: u32,
    pub jump_per_layer: u32,
    pub dynamic_wait_after_lift: String,
    pub dynamic_lift: String,
    pub jump_height: u32,
    pub low_quality_cure_time: u32,
    pub low_quality_skip_per_layer: u32,
    #[serde(rename = "XYResPerc")]
    pub xy_res_perc: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct ProfileCustomValues {
    cd_threshold: String,
    dw_flow_end_slope: String,
    fss_enable_crashdetection: String,
    fss_enable_dynamicwait: String,
    fss_enable_peeldetection: String,
    fss_enable_resinleveldetection: String,
    pd_peel_end_slope: String,
    pd_peel_start_slope: String,
    resin_preheat_temperature: String,
    rl_threshold: String,
    uv_pwm_value: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            format_version: 2,
            distro: "generic".into(),
            program: "mslicer".into(),
            version: Default::default(),
            os: env::consts::OS.into(),
            arch: env::consts::ARCH.into(),
            profile: false,
        }
    }
}

impl Default for ProfileCustomValues {
    fn default() -> Self {
        ProfileCustomValues {
            cd_threshold: "2000".into(),
            dw_flow_end_slope: "10".into(),
            fss_enable_crashdetection: "0".into(),
            fss_enable_dynamicwait: "0".into(),
            fss_enable_peeldetection: "0".into(),
            fss_enable_resinleveldetection: "0".into(),
            pd_peel_end_slope: "0".into(),
            pd_peel_start_slope: "0".into(),
            resin_preheat_temperature: "25".into(),
            rl_threshold: "-5".into(),
            uv_pwm_value: "1".into(),
        }
    }
}
