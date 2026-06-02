use std::{fs::File, io::BufReader, iter, path::Path, sync::Arc};

use common::{
    progress::Progress,
    slice::{ExposureConfig, Layer, SliceConfig},
    units::{Milimeter, Milimeters, Minutes, Seconds},
};
use gerber_parser::{
    GerberDoc,
    gerber_types::{Command, ExtendedCode, FileAttribute, FileFunction},
};
use itertools::Itertools;
use nalgebra::Vector2;

pub use misc::Alignment;
use polygons::Polygons;
use slicer::slicer::raster;

mod gerber;
mod misc;
mod polygons;

#[derive(Clone)]
pub struct PrintedCircuitBoard {
    pub gerber: Option<Arc<Gerber>>,
    pub alignment: Alignment,
    pub flip: Flip,
    pub offset: Vector2<Milimeters>,
    pub exposure_time: Seconds,
    pub invert: bool,
}

pub struct Gerber {
    document: GerberDoc,
    pub name: Option<String>,
    pub layer: Option<String>,
}

#[derive(Clone, Default)]
pub struct Flip {
    pub enabled: bool,
    pub angle: f32,
    pub alignment: Alignment,
    pub offset: Milimeters,
}

impl PrintedCircuitBoard {
    pub fn slice_config(&self, config: &mut SliceConfig) {
        config.first_exposure_config.exposure_time = self.exposure_time;
        config.exposure_config.exposure_time = self.exposure_time;
    }

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let platform = config.platform_resolution;
        let segments = self.screen_segments(config, self.polygons(progress));
        let mut runs = raster::layer(config.supersample, platform, segments.into_iter());

        if self.invert {
            runs.iter_mut().for_each(|run| run.value = 255 - run.value);
        }

        vec![Layer {
            data: runs,
            exposure: ExposureConfig {
                exposure_time: self.exposure_time,
                exposure_delay: Seconds::new(0.0),
                pwm: 255,
                ..config.exposure_config(0).into_owned()
            },
        }]
    }

    pub fn load(&mut self, path: &Path) {
        let reader = BufReader::new(File::open(path).unwrap());
        let document = gerber_parser::parse(reader).unwrap();

        let mut name = None;
        let mut layer = None;
        for command in document.commands.iter() {
            if let Ok(Command::ExtendedCode(ExtendedCode::FileAttribute(attr))) = command {
                match attr {
                    FileAttribute::FileFunction(function) => {
                        layer = Some(match function {
                            FileFunction::Copper { pos, .. } => format!("{pos:?} Copper"),
                            FileFunction::Legend { pos, .. } => format!("{pos:?} Silkscreen"),
                            FileFunction::SolderMask { pos, .. } => format!("{pos:?} Solder Mask"),
                            FileFunction::Paste(pos) => format!("{pos:?} Solder Paste"),
                            x => format!("{x:?}"),
                        })
                    }
                    FileAttribute::ProjectId { id, .. } => name = Some(id.to_owned()),
                    _ => {}
                }
            }
        }

        self.gerber = Some(Arc::new(Gerber {
            document,
            name,
            layer,
        }));
    }

    pub fn svg(&self) -> String {
        self.polygons(&Progress::new()).svg()
    }

    fn screen_segments(
        &self,
        config: &SliceConfig,
        mut polygons: Polygons,
    ) -> Vec<(([Vector2<f32>; 2], bool), u8)> {
        let platform = (config.platform_size.xy()).map(|x| x.get::<Milimeter>() as f64);
        let scale = (config.platform_resolution.cast::<f64>()).component_div(&platform);

        let offset = self.alignment.offset(platform, polygons.bounds())
            + self.offset.map(|x| x.get::<Milimeter>() as f64);
        polygons.transform_mut(offset);

        if self.flip.enabled {
            let (sin, cos) = (self.flip.angle as f64).to_radians().sin_cos();
            let anchor = self.flip.alignment.offset(platform, [Vector2::zeros(); 2]);

            let normal = Vector2::new(sin, -cos);
            let center = anchor + normal * (self.flip.offset.get::<Milimeter>() as f64);

            for polygon in polygons.polygons.iter_mut() {
                for point in polygon.iter_mut() {
                    let distance = (*point - center).dot(&normal);
                    *point -= 2.0 * distance * normal;
                }
            }
        }

        polygons.nonuniform_scale_mut(scale * config.supersample as f64); // screen space to pixel space

        let mut out = Vec::new();
        for polygon in polygons.polygons.iter_mut() {
            let winding = winding_order(polygon);

            let close = (polygon.last().unwrap(), polygon.first().unwrap());
            for (&a, &b) in polygon.iter().tuple_windows().chain(iter::once(close)) {
                let segment = [a, b].map(|x| x.cast());
                let normal = (b.y - a.y) * winding > 0.0;
                out.push(((segment, normal), 255));
            }
        }

        out
    }
}

// Reference: https://stackoverflow.com/a/1180256
fn winding_order(polygon: &[Vector2<f64>]) -> f64 {
    // Find a point on the convex hull
    let min = (polygon.iter())
        .position_min_by(|a, b| a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)))
        .unwrap();

    let a = polygon[(min + polygon.len() - 1) % polygon.len()];
    let b = polygon[min];
    let c = polygon[(min + 1) % polygon.len()];

    (b - a).perp(&(c - a)).signum()
}

impl Default for PrintedCircuitBoard {
    fn default() -> Self {
        Self {
            gerber: Default::default(),
            alignment: Default::default(),
            flip: Default::default(),
            offset: Default::default(),
            exposure_time: Minutes::new(5.0).convert(),
            invert: Default::default(),
        }
    }
}
