use std::f32::consts::PI;

use common::{
    container::Run,
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use nalgebra::Vector3;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Default, Clone)]
pub struct SdfSlicer {}

impl SdfSlicer {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let px = config.pixel_size().map(|x| x.get::<Milimeter>());
        let slice_height = config.slice_height.get::<Milimeter>();
        let size = config.platform_size.map(|x| x.get::<Milimeter>());
        let offset = Vector3::new(size.x / 2.0, size.y / 2.0, 0.0);

        let layers = (config.platform_size.z / config.slice_height) as u32;
        let rows = config.platform_resolution.y;

        progress.set_total(layers as u64);
        (0..layers)
            .into_par_iter()
            .map(|i| {
                let pz = i as f32 * slice_height;
                let mut runs = Vec::new();

                for y in 0..rows {
                    let py = y as f32 * px.y;
                    let mut x = 0;
                    while x < config.platform_resolution.x {
                        let pos = Vector3::new(x as f32 * px.x, py, pz) - offset;
                        let dist = sdf(pos);
                        let dist_px = dist / px.x;

                        if dist_px.abs() < 1.0 {
                            runs.push(Run::new(1, ((1.0 - dist_px) * 127.5) as u8));
                            x += 1;
                        } else {
                            let length =
                                (dist_px.abs() as u32).min(config.platform_resolution.x - x);
                            runs.push(Run::new(length as u64, [0, 255][(dist < 0.0) as usize]));
                            x += length as u32;
                        }
                    }
                }

                Layer::new(
                    runs,
                    config.default_height(i),
                    config.exposure_config(i).into_owned(),
                )
            })
            .inspect(|_| progress.add_complete(1))
            .collect()
    }
}

// From: https://jbaker.graphics/writings/DEC.html
fn sdf(mut p: Vector3<f32>) -> f32 {
    let rt = 15.0;
    let rg = 4.0;
    let ws = 0.3;

    let nx = rt * p.z.atan2(-p.x);
    let nz = p.xz().magnitude() - rt;
    p.x = nx;
    p.z = nz;

    let ny = rg * p.z.atan2(-p.y);
    let nz = p.yz().magnitude() - rg;
    p.y = ny;
    p.z = nz;

    let s = p.map(|x| x.sin()).dot(&p.map(|x| x.cos()).yzx());
    0.6 * ((s.abs() - ws).max(p.z.abs() - 0.5 * PI))
}
