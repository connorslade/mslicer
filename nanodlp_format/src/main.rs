use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

fn main() -> Result<()> {
    let path = "/home/connorslade/Downloads/dragon.nanodlp";
    let file = fs::File::open(path)?;
    let mut zip = ZipArchive::new(file)?;

    let meta = serde_json::from_reader::<_, Meta>(zip.by_name("meta.json")?)?;
    let layers = serde_json::from_reader::<_, Vec<LayerInfo>>(zip.by_name("info.json")?)?;
    let plate = serde_json::from_reader::<_, Plate>(zip.by_name("plate.json")?)?;
    let slicer = serde_json::from_reader::<_, Slicer>(zip.by_name("slicer.json")?)?;
    let profile = serde_json::from_reader::<_, Profile>(zip.by_name("profile.json")?)?;

    let file = File {
        meta,
        layers,
        plate,
        slicer,
        profile,
    };

    println!("{file:?}");

    Ok(())
}

#[derive(Debug)]
#[allow(unused)] // temporary
struct File {
    meta: Meta,
    layers: Vec<LayerInfo>,
    plate: Plate,
    slicer: Slicer,
    profile: Profile,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Meta {
    format_version: u32,
    distro: String,
    program: String,
    version: String,
    #[serde(rename = "OS")]
    os: String,
    arch: String,
    profile: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LayerInfo {
    total_solid_area: f32,
    largest_area: f32,
    smallest_area: f32,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
    area_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Plate {
    #[serde(rename = "PlateID")]
    plate_id: u32,
    #[serde(rename = "ProfileID")]
    profile_id: u32,
    created_date: u32,
    stop_layers: String,
    path: String,
    low_quality_layer_number: u32,
    auto_center: u32,
    updated: u32,
    last_print: u32,
    print_time: u32,
    print_est: u32,
    image_rotate: u32,
    mask_effect: u32,
    x_res: u32,
    y_res: u32,
    z_res: u32,
    multi_cure: String,
    multi_thickness: String,
    offset: u32,
    risky: bool,
    is_faulty: bool,
    repaired: bool,
    corrupted: bool,
    total_solid_area: f32,
    layers_count: u32,
    feedback: bool,
    re_slice_needed: bool,
    multi_material: bool,
    #[serde(rename = "PrintID")]
    print_id: u32,
    #[serde(rename = "MC")]
    mc: PlateMc,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    z_min: f32,
    z_max: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PlateMc {
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    multi_cure_gap: u32,
    count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Slicer {
    _type: String,
    #[serde(rename = "URL")]
    url: String,
    p_width: u32,
    p_height: u32,
    scale_factor: u32,
    start_layer: u32,
    support_depth: u32,
    support_layer_number: u32,
    thickness: u32,
    x_offset: u32,
    y_offset: u32,
    z_offset: u32,
    x_pixel_size: f32,
    y_pixel_size: f32,
    // mask: null,
    slice_from_zero: bool,
    disable_validator: bool,
    auto_center: u32,
    preview_generate: bool,
    running: bool,
    debug: bool,
    is_faulty: bool,
    corrupted: bool,
    multi_material: bool,
    adapt_export: String,
    preview_color: String,
    // faulty_layers: [],
    // overhang_layers: [],
    // layer_status: [],
    // "Boundary": {
    //   "XMin": -71.72,
    //   "XMax": 71.73,
    //   "YMin": -31.39,
    //   "YMax": 31.42,
    //   "ZMin": 0,
    //   "ZMax": 97
    // },
    #[serde(rename = "MC")]
    mc: PlateMc,
    multi_thickness: String,
    export_path: String,
    network_save: String,
    file: String,
    file_size: u32,
    adapt_slicing: u32,
    adapt_slicing_min: u32,
    adapt_slicing_max: u32,
    support_offset: u32,
    offset: u32,
    fill_color: String,
    blank_color: String,
    dim_amount: u32,
    dim_wall: u32,
    dim_skip: u32,
    pixel_diming: u32,
    hatching_type: u32,
    elephant_mid_exposure: u32,
    elephant_type: u32,
    elephant_amount: u32,
    elephant_wall: u32,
    elephant_layers: u32,
    hatching_wall: u32,
    hatching_gap: u32,
    hatching_outer_wall: u32,
    hatching_top_cap: u32,
    hatching_bottom_cap: u32,
    multi_cure_gap: u32,
    anti_alias: u32,
    anti_alias_3_d: u32,
    anti_alias_3_d_distance: u32,
    anti_alias_3_d_min: u32,
    anti_alias_threshold: u32,
    image_rotate: u32,
    ignore_mask: u32,
    #[serde(rename = "XYRes")]
    xy_res: u32,
    x_res: u32,
    y_res: u32,
    z_res_perc: u32,
    preview_width: u32,
    preview_height: u32,
    barrel_factor: u32,
    barrel_x: u32,
    barrel_y: u32,
    image_mirror: u32,
    display_controller: u32,
    light_output_formula: String,
    #[serde(rename = "PlateID")]
    plate_id: u32,
    #[serde(rename = "LayerID")]
    layer_id: u32,
    layer_count: u32,
    #[serde(rename = "UUID")]
    uuid: String,
    // dynamic_thickness: [],
    #[serde(rename = "FillColorRGB")]
    fill_color_rgb: Color,
    #[serde(rename = "BlankColorRGB")]
    blank_color_rgb: Color,
    export_type: u32,
    output_path: String,
    suffix: String,
    skip_empty: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Color {
    r: u32,
    g: u32,
    b: u32,
    a: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Profile {
    #[serde(rename = "ResinID")]
    resin_id: u32,
    #[serde(rename = "ProfileID")]
    profile_id: u32,
    title: String,
    desc: String,
    color: String,
    resin_price: u32,
    optimum_temperature: u32,
    depth: u32,
    support_top_wait: u32,
    support_wait_height: u32,
    support_depth: u32,
    support_wait_before_print: u32,
    support_wait_after_print: u32,
    transitional_layer: u32,
    updated: u32,
    custom_values: ProfileCustomValues,
    _type: u32,
    z_step_wait: u32,
    top_wait: u32,
    wait_height: u32,
    cure_time: u32,
    wait_before_print: u32,
    wait_after_print: u32,
    support_cure_time: u32,
    support_layer_number: u32,
    adapt_slicing: u32,
    adapt_slicing_min: u32,
    adapt_slicing_max: u32,
    support_offset: u32,
    offset: u32,
    fill_color: String,
    blank_color: String,
    dim_amount: u32,
    dim_wall: u32,
    dim_skip: u32,
    pixel_diming: u32,
    hatching_type: u32,
    elephant_mid_exposure: u32,
    elephant_type: u32,
    elephant_amount: u32,
    elephant_wall: u32,
    elephant_layers: u32,
    hatching_wall: u32,
    hatching_gap: u32,
    hatching_outer_wall: u32,
    hatching_top_cap: u32,
    hatching_bottom_cap: u32,
    multi_cure_gap: u32,
    anti_alias: u32,
    anti_alias_3_d: u32,
    anti_alias_3_d_distance: u32,
    anti_alias_3_d_min: u32,
    anti_alias_threshold: u32,
    image_rotate: u32,
    ignore_mask: u32,
    #[serde(rename = "XYRes")]
    xy_res: u32,
    y_res: u32,
    z_res_perc: u32,
    dynamic_cure_time: String,
    dynamic_speed: String,
    shield_before_layer: String,
    shield_after_layer: String,
    shield_during_cure: String,
    shield_start: String,
    shield_resume: String,
    shield_finish: String,
    laser_code: String,
    shutter_open_gcode: String,
    shutter_close_gcode: String,
    separation_detection: String,
    resin_level_detection: String,
    crash_detection: String,
    dynamic_wait: String,
    slow_section_height: u32,
    slow_section_step_wait: u32,
    jump_per_layer: u32,
    dynamic_wait_after_lift: String,
    dynamic_lift: String,
    jump_height: u32,
    low_quality_cure_time: u32,
    low_quality_skip_per_layer: u32,
    #[serde(rename = "XYResPerc")]
    xy_res_perc: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ProfileCustomValues {
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
