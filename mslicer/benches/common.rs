use ::common::{
    annotations::Annotations,
    config::{ExposureConfig, SliceConfig},
};
use goo_format::File;

use mslicer::app::slice_operation::SliceResult;
use nalgebra::{Vector2, Vector3};
use slicer::format::FormatSliceFile;

pub fn prepare() -> (SliceConfig, SliceResult) {
    let benchfile = std::env::var("MSLICER_BENCHMARK_OBJECT")
        .expect("must set MSLICER_BENCHMARK_OBJECT env var to path to .goo file");
    let data = std::fs::read(benchfile);
    let file = File::deserialize(data.unwrap().as_slice()).unwrap();
    let header = file.header.clone();
    let fsf = FormatSliceFile::Goo(Box::new(file));
    let normal_exp_cfg = ExposureConfig {
        exposure_time: header.exposure_time,
        lift_distance: header.lift_distance,
        lift_speed: header.lift_speed,
        retract_distance: header.retract_distance,
        retract_speed: header.retract_speed,
    };
    let bottom_exp_cfg = ExposureConfig {
        exposure_time: header.bottom_exposure_time,
        lift_distance: header.bottom_lift_distance,
        lift_speed: header.bottom_lift_speed,
        retract_distance: header.bottom_retract_distance,
        retract_speed: header.bottom_retract_speed,
    };
    let cfg: SliceConfig = SliceConfig {
        format: common::format::Format::Goo,
        platform_resolution: Vector2::new(header.x_resolution.into(), header.y_resolution.into()),
        platform_size: Vector3::new(header.x_size, header.y_size, header.z_size),
        slice_height: header.layer_thickness,
        exposure_config: normal_exp_cfg,
        first_exposure_config: bottom_exp_cfg,
        first_layers: header.bottom_layers,
        transition_layers: header.transition_layers as u32,
    };
    let layers = fsf.info().layers as usize;
    let slice_res = mslicer::app::slice_operation::SliceResult {
        file: fsf,
        slice_preview_layer: 0,
        last_preview_layer: 0,
        preview_offset: Vector2::new(0.0, 0.0),
        preview_scale: 1.0_f32.log2(),
        layer_count: (layers, layers.to_string().len() as u8),
        annotations: Annotations::default(),
        show_error_annotations: true,
        show_warning_annotations: true,
        show_info_annotations: true,
        show_debug_annotations: false,
    };
    (cfg, slice_res)
}
