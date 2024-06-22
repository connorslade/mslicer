use nalgebra::{Vector2, Vector3};
use ordered_float::OrderedFloat;

use crate::mesh::Mesh;
use goo_format::{File as GooFile, HeaderInfo, LayerContent, LayerEncoder};

#[derive(Clone, Debug)]
pub struct SliceConfig {
    pub platform_resolution: Vector2<u32>,
    pub platform_size: Vector3<f32>,
    pub slice_height: f32,

    pub exposure_config: ExposureConfig,
    pub first_exposure_config: ExposureConfig,
    pub first_layers: u32,
}

#[derive(Clone, Debug)]
pub struct ExposureConfig {
    pub exposure_time: f32,
    pub lift_distance: f32,
    pub lift_speed: f32,
    pub retract_distance: f32,
    pub retract_speed: f32,
}

pub fn slice_goo(slice_config: &SliceConfig, modal: &Mesh, progress: impl Fn(u32, u32)) -> GooFile {
    let (_, max) = modal.minmax_point();
    let max = modal.transform(&max);
    let layers = (max.z / slice_config.slice_height).ceil() as u32;

    let layers = (0..layers)
        .inspect(|&layer| {
            progress(layer, layers);
        })
        .map(|layer| {
            let height = layer as f32 * slice_config.slice_height;
            let intersections = modal.intersect_plane(height);

            let segments = intersections
                .chunks(2)
                .map(|x| (x[0], x[1]))
                .collect::<Vec<_>>();

            let mut out = Vec::new();
            for y in 0..slice_config.platform_resolution.y {
                let mut intersections = segments
                    .iter()
                    .filter_map(|(a, b)| {
                        let y = y as f32;
                        if a.y <= y && b.y >= y {
                            let t = (y - a.y) / (b.y - a.y);
                            let x = a.x + t * (b.x - a.x);
                            Some(x)
                        } else if b.y <= y && a.y >= y {
                            let t = (y - b.y) / (a.y - b.y);
                            let x = b.x + t * (a.x - b.x);
                            Some(x)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                intersections.sort_by_key(|&x| OrderedFloat(x));
                intersections.dedup();

                for span in intersections.chunks_exact(2) {
                    let y_offset = (slice_config.platform_resolution.x * y) as u64;
                    out.push((y_offset + span[0] as u64, y_offset + span[1] as u64));
                }
            }

            let mut encoder = LayerEncoder::new();

            let mut last = 0;
            for (start, end) in out {
                if start > last {
                    encoder.add_run(start - last, 0);
                }

                encoder.add_run(end - start, 255);
                last = end;
            }

            let image_size = slice_config.platform_resolution.x as u64
                * slice_config.platform_resolution.y as u64;
            encoder.add_run(image_size - last, 0);

            let (data, checksum) = encoder.finish();
            let layer_exposure = if layer < slice_config.first_layers {
                &slice_config.first_exposure_config
            } else {
                &slice_config.exposure_config
            };

            LayerContent {
                data,
                checksum,
                layer_position_z: slice_config.slice_height * (layer + 1) as f32,

                layer_exposure_time: layer_exposure.exposure_time,
                lift_distance: layer_exposure.lift_distance,
                lift_speed: layer_exposure.lift_speed,
                retract_distance: layer_exposure.retract_distance,
                retract_speed: layer_exposure.retract_speed,
                pause_position_z: slice_config.platform_size.z,
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();

    let layer_time = slice_config.exposure_config.exposure_time
        + slice_config.exposure_config.lift_distance / slice_config.exposure_config.lift_speed;
    let bottom_layer_time = slice_config.first_exposure_config.exposure_time
        + slice_config.first_exposure_config.lift_distance
            / slice_config.first_exposure_config.lift_speed;
    let total_time = (layers.len() as u32 - slice_config.first_layers) as f32 * layer_time
        + slice_config.first_layers as f32 * bottom_layer_time;

    GooFile::new(
        HeaderInfo {
            x_resolution: slice_config.platform_resolution.x as u16,
            y_resolution: slice_config.platform_resolution.y as u16,
            x_size: slice_config.platform_size.x,
            y_size: slice_config.platform_size.y,

            layer_count: layers.len() as u32,
            printing_time: total_time as u32,
            layer_thickness: slice_config.slice_height,
            bottom_layers: slice_config.first_layers,
            transition_layers: slice_config.first_layers as u16 + 1,

            exposure_time: slice_config.exposure_config.exposure_time,
            lift_distance: slice_config.exposure_config.lift_distance,
            lift_speed: slice_config.exposure_config.lift_speed,
            retract_distance: slice_config.exposure_config.retract_distance,
            retract_speed: slice_config.exposure_config.retract_speed,

            bottom_exposure_time: slice_config.first_exposure_config.exposure_time,
            bottom_lift_distance: slice_config.first_exposure_config.lift_distance,
            bottom_lift_speed: slice_config.first_exposure_config.lift_speed,
            bottom_retract_distance: slice_config.first_exposure_config.retract_distance,
            bottom_retract_speed: slice_config.first_exposure_config.retract_speed,

            ..Default::default()
        },
        layers,
    )
}

impl Default for ExposureConfig {
    fn default() -> Self {
        Self {
            exposure_time: 3.0,
            lift_distance: 5.0,
            lift_speed: 65.0,
            retract_distance: 5.0,
            retract_speed: 150.0,
        }
    }
}
