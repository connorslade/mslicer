use std::{fs::File, io::BufReader, iter, mem, path::Path, sync::Arc};

use common::{
    progress::Progress,
    slice::{ExposureConfig, Layer, SliceConfig},
    units::{Milimeter, Milimeters, Minutes, Seconds},
};
use gerber_parser::{
    GerberDoc,
    gerber_types::{
        Aperture, Command, DCode, ExtendedCode, FileAttribute, FileFunction, FunctionCode, GCode,
        MacroDecimal, Operation,
    },
};
use itertools::Itertools;
use nalgebra::Vector2;

pub use misc::Alignment;
use polygons::Polygons;
use slicer::slicer::raster;

mod misc;
mod polygons;

#[derive(Clone)]
pub struct PrintedCircuitBoard {
    pub gerber: Option<Arc<Gerber>>,
    pub alignment: Alignment,
    pub offset: Vector2<Milimeters>,
    pub exposure_time: Seconds,
    pub invert: bool,
}

pub struct Gerber {
    document: GerberDoc,
    pub name: Option<String>,
    pub layer: Option<String>,
}

impl PrintedCircuitBoard {
    pub fn slice_config(&self, config: &mut SliceConfig) {
        config.first_exposure_config.exposure_time = self.exposure_time;
        config.exposure_config.exposure_time = self.exposure_time;
    }

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let gerber = &self.gerber.as_ref().unwrap().document;
        let commands = gerber.commands();
        progress.set_total(commands.len() as u64);

        let mut aperture: Option<&Aperture> = None;
        let mut thickness = 0.0;

        let mut polygons = Polygons::new();
        let mut path = Vec::new();
        let mut region_mode = false;

        for command in commands {
            match command {
                Command::FunctionCode(function_code) => match function_code {
                    FunctionCode::DCode(dcode) => match dcode {
                        DCode::Operation(Operation::Move(Some(mov))) => {
                            flush_path(&mut polygons, &mut path, region_mode, thickness);
                            let point = Vector2::new(mov.x, mov.y).map(|x| x.unwrap().into());
                            path.push(point);
                        }
                        DCode::Operation(Operation::Interpolate(Some(pos), _offset)) => {
                            let point = Vector2::new(pos.x, pos.y).map(|x| x.unwrap().into());
                            path.push(point);
                        }
                        DCode::SelectAperture(x) => {
                            flush_path(&mut polygons, &mut path, region_mode, thickness);
                            aperture = gerber.apertures.get(x);
                            if let Some(aperture) = aperture {
                                thickness = match aperture {
                                    Aperture::Circle(circle) => circle.diameter,
                                    Aperture::Rectangle(rect) => rect.x.min(rect.y),
                                    Aperture::Obround(obround) => obround.x.min(obround.y),
                                    _ => thickness,
                                };
                            }
                        }
                        DCode::Operation(Operation::Flash(Some(flash))) => {
                            flush_path(&mut polygons, &mut path, region_mode, thickness);

                            let center = Vector2::new(flash.x, flash.y).map(|x| x.unwrap().into());
                            match aperture {
                                Some(Aperture::Circle(circle)) => {
                                    polygons.circle(center, circle.diameter / 2.0);
                                }
                                Some(Aperture::Rectangle(rect)) => {
                                    let rect = Vector2::new(rect.x, rect.y) / 2.0;
                                    polygons.rect([center - rect, center + rect]);
                                }
                                Some(Aperture::Obround(rect)) => {
                                    let rect = Vector2::new(rect.x, rect.y);
                                    let radius = rect.x.min(rect.y) / 2.0;
                                    let mut circle = |x: f64, y: f64| {
                                        polygons.circle(center + Vector2::new(x, y), radius)
                                    };

                                    let size = if rect.x >= rect.y {
                                        let dx = rect.x / 2.0 - radius;
                                        circle(-dx, 0.0);
                                        circle(dx, 0.0);
                                        Vector2::new(dx, radius)
                                    } else {
                                        let dy = rect.y / 2.0 - radius;
                                        circle(0.0, -dy);
                                        circle(0.0, dy);
                                        Vector2::new(radius, dy)
                                    };

                                    polygons.rect([center - size, center + size]);
                                }
                                Some(Aperture::Macro(name, Some(params))) => {
                                    // [radius, x1, y1, x2, y2, x3, y3, x4, y4]
                                    if name == "RoundRect" {
                                        let radius = macro_value(&params[0]);
                                        let corners = [0, 1, 2, 3].map(|i| {
                                            Vector2::new(i * 2 + 1, i * 2 + 2)
                                                .map(|x| macro_value(&params[x]))
                                                + center
                                        });
                                        polygons.rounded_rect(corners, radius);
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    FunctionCode::GCode(GCode::RegionMode(mode)) => {
                        flush_path(&mut polygons, &mut path, region_mode, thickness);
                        region_mode = *mode;
                    }
                    _ => {}
                },
                _ => {}
            }

            progress.add_complete(1);
        }

        flush_path(&mut polygons, &mut path, region_mode, thickness);
        progress.set_finished();

        let platform = config.platform_resolution;
        let segments = self.screen_segments(config, polygons);
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

    fn screen_segments(
        &self,
        config: &SliceConfig,
        mut polygons: Polygons,
    ) -> Vec<(([Vector2<f32>; 2], bool), u8)> {
        let platform_size = (config.platform_size.xy()).map(|x| x.get::<Milimeter>() as f64);
        let scale = (config.platform_resolution.cast::<f64>()).component_div(&platform_size);

        polygons.nonuniform_scale_mut(scale * config.supersample as f64);
        let offset = self.offset(config, polygons.bounds())
            + config
                .mm_to_px(self.offset.map(|x| x.get::<Milimeter>()))
                .cast();

        let mut out = Vec::new();
        for polygon in polygons.polygons.iter() {
            let winding = winding_order(polygon);

            let close = (polygon.last().unwrap(), polygon.first().unwrap());
            for (&a, &b) in polygon.iter().tuple_windows().chain(iter::once(close)) {
                let segment = [a, b].map(|x| (x + offset).map(|x| x as f32));
                let normal = (b.y - a.y) * winding > 0.0;
                out.push(((segment, normal), 255));
            }
        }

        out
    }

    fn offset(&self, config: &SliceConfig, [min, max]: [Vector2<f64>; 2]) -> Vector2<f64> {
        let platform = config.platform_resolution.cast() * config.supersample as f64;
        match self.alignment {
            Alignment::TopLeft => Vector2::new(-min.x, -min.y),
            Alignment::TopRight => Vector2::new(platform.x - max.x, -min.y),
            Alignment::BottomLeft => Vector2::new(-min.x, platform.y - max.y),
            Alignment::BottomRight => Vector2::new(platform.x - max.x, platform.y - max.y),
            Alignment::Center => Vector2::new(
                (platform.x - (min.x + max.x)) / 2.0,
                (platform.y - (min.y + max.y)) / 2.0,
            ),
        }
    }
}

// Reference: https://stackoverflow.com/a/1180256
fn winding_order(polygon: &[Vector2<f64>]) -> f64 {
    let min = (polygon.iter())
        .position_min_by(|a, b| a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)))
        .unwrap();

    let a = polygon[(min + polygon.len() - 1) % polygon.len()];
    let b = polygon[min];
    let c = polygon[(min + 1) % polygon.len()];

    let ab = b - a;
    let ac = c - a;
    ab.perp(&ac).signum()
}

fn flush_path(
    polygons: &mut Polygons,
    path: &mut Vec<Vector2<f64>>,
    region_mode: bool,
    thickness: f64,
) {
    if path.is_empty() {
        return;
    }

    if region_mode {
        polygons.trace(mem::take(path), None);
    } else {
        polygons.trace(mem::take(path), Some(thickness));
    }
}

fn macro_value(value: &MacroDecimal) -> f64 {
    match value {
        MacroDecimal::Value(value) => *value,
        _ => 0.0,
    }
}

impl Default for PrintedCircuitBoard {
    fn default() -> Self {
        Self {
            gerber: Default::default(),
            alignment: Default::default(),
            offset: Default::default(),
            exposure_time: Minutes::new(3.0).convert(),
            invert: Default::default(),
        }
    }
}
