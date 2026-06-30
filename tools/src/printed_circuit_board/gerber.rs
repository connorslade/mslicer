use std::mem;

use common::progress::Progress;
use gerber_parser::gerber_types::{
    Aperture, Command, DCode, FunctionCode, GCode, MacroDecimal, Operation,
};
use nalgebra::Vector2;
use tracing::warn;

use crate::printed_circuit_board::{Gerber, polygons::Polygons};

pub fn tessellate(gerber: &Gerber, polygons: &mut Polygons, progress: &Progress) {
    let gerber = &gerber.document;
    let commands = gerber.commands();

    let mut aperture: Option<&Aperture> = None;
    let mut thickness = 0.0;

    let mut path = Vec::new();
    let mut region_mode = false;

    for command in commands {
        if let Command::FunctionCode(function_code) = command {
            match function_code {
                FunctionCode::DCode(dcode) => match dcode {
                    DCode::Operation(Operation::Move(Some(mov))) => {
                        flush_path(polygons, &mut path, region_mode, thickness);
                        let point = Vector2::new(mov.x, mov.y).map(|x| x.unwrap().into());
                        path.push(point);
                    }
                    DCode::Operation(Operation::Interpolate(Some(pos), _offset)) => {
                        let point = Vector2::new(pos.x, pos.y).map(|x| x.unwrap().into());
                        path.push(point);
                    }
                    DCode::SelectAperture(x) => {
                        flush_path(polygons, &mut path, region_mode, thickness);
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
                        flush_path(polygons, &mut path, region_mode, thickness);

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
                                match name.as_str() {
                                    "RoundRect" => {
                                        // [radius, x1, y1, x2, y2, x3, y3, x4, y4]
                                        let radius = macro_value(&params[0]);
                                        let corners = [0, 1, 2, 3].map(|i| {
                                            Vector2::new(i * 2 + 1, i * 2 + 2)
                                                .map(|x| macro_value(&params[x]))
                                                + center
                                        });
                                        polygons.rounded_rect(corners, radius);
                                    }
                                    x => warn!("Unknown aperture macro: {x}"),
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                FunctionCode::GCode(GCode::RegionMode(mode)) => {
                    flush_path(polygons, &mut path, region_mode, thickness);
                    region_mode = *mode;
                }
                _ => {}
            }
        }

        progress.add_complete(1);
    }

    flush_path(polygons, &mut path, region_mode, thickness);
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
