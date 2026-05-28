use std::{fs::File, io::BufReader, path::PathBuf};

use common::{
    container::Image,
    progress::Progress,
    slice::{ExposureConfig, Layer, SliceConfig},
    units::{Milimeter, Minutes, Seconds},
};
use gerber_parser::gerber_types::{Aperture, Command, Coordinates, DCode, FunctionCode, Operation};
use nalgebra::Vector2;

#[derive(Clone)]
pub struct PrintedCircuitBoard {
    pub gerber: Option<PathBuf>,
    pub alignment: Alignment,
    pub exposure_time: Seconds,
    pub invert: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

impl PrintedCircuitBoard {
    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let file = File::open(self.gerber.as_ref().unwrap()).unwrap();
        let reader = BufReader::new(file);

        let gerber = gerber_parser::parse(reader).unwrap();
        let commands = gerber.commands();

        progress.set_total(commands.len() as u64);
        let offset = self.offset(config, bounds(&commands));

        let mut image = Image::blank(config.platform_resolution.cast());

        let mut aperture: Option<&Aperture> = None;
        let mut thickness = 0.0;
        for command in commands {
            let Command::FunctionCode(FunctionCode::DCode(code)) = command else {
                continue;
            };

            match code {
                DCode::Operation(Operation::Move(mov)) => {
                    // if !path.is_empty() {
                    //     shapes.push(close_path(mem::take(&mut path), thickness));
                    // }

                    // let point = mov.into();
                    // shapes.push(generate_circle(point, thickness / 2.0, CIRCLE_SIDES));
                    // path.push(point);
                }
                DCode::Operation(Operation::Interpolate(pos, _offset)) => {
                    // let point = pos.into();
                    // shapes.push(generate_circle(point, thickness / 2.0, CIRCLE_SIDES));
                    // path.push(point);
                }

                DCode::SelectAperture(x) => {
                    aperture = gerber.apertures.get(&x);
                    if let Some(Aperture::Circle(circle)) = aperture {
                        thickness = circle.diameter;
                    }
                }
                DCode::Operation(Operation::Flash(Some(flash))) => {
                    let flash = Vector2::new(flash.x.unwrap().into(), flash.y.unwrap().into());
                    let pos = self.mm_to_px(config, flash + offset);
                    match aperture {
                        Some(Aperture::Circle(circle)) => {
                            let r = self.mm_to_px(config, Vector2::repeat(circle.diameter / 2.0));
                            image.circle(pos, r.cast(), 255);
                        }
                        Some(Aperture::Rectangle(rect)) => {
                            let rect = self.mm_to_px(config, Vector2::new(rect.x, rect.y));
                            image.rect((pos - rect, pos + rect), 255);
                        }
                        Some(Aperture::Obround(rect)) => {
                            // let rect = Point::new(rect.x, rect.y) * config.aperture_thickness;
                            // let radius = rect.x.min(rect.y) / 2.0;
                            // let mut circle = |offset: Point| {
                            //     shapes.push(generate_circle(pos + offset, radius, CIRCLE_SIDES));
                            // };

                            // let size = if rect.y < rect.x {
                            //     circle(Point::new(-rect.x / 2.0 + radius, 0.0));
                            //     circle(Point::new(rect.x / 2.0 - radius, 0.0));
                            //     Point::new(rect.x - rect.y, rect.y)
                            // } else {
                            //     circle(Point::new(0.0, -rect.x / 2.0 + radius));
                            //     circle(Point::new(0.0, rect.x / 2.0 - radius));
                            //     Point::new(rect.x, rect.x)
                            // };
                            // shapes.push(generate_rectangle(pos, size));
                        }
                        _ => {}
                    }
                }
                _ => {}
            };
            progress.add_complete(1);
        }

        progress.set_finished();
        vec![Layer {
            data: image.runs().collect::<Vec<_>>(),
            exposure: ExposureConfig {
                exposure_time: self.exposure_time,
                exposure_delay: Seconds::new(0.0),
                pwm: 255,
                ..config.exposure_config(0).into_owned()
            },
        }]
    }

    fn offset(&self, config: &SliceConfig, [min, max]: [Vector2<f64>; 2]) -> Vector2<f64> {
        let platform = config
            .platform_size
            .map(|x| x.get::<Milimeter>())
            .cast::<f64>();

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

    fn mm_to_px(&self, config: &SliceConfig, mm: Vector2<f64>) -> Vector2<usize> {
        config.mm_to_px(mm.cast()).map(|x| x.round() as usize)
    }
}

fn bounds(commands: &[&Command]) -> [Vector2<f64>; 2] {
    let (mut min, mut max) = (
        Vector2::repeat(f64::INFINITY),
        Vector2::repeat(f64::NEG_INFINITY),
    );
    let mut pos = Vector2::zeros();

    for command in commands {
        let Command::FunctionCode(FunctionCode::DCode(code)) = command else {
            continue;
        };

        let coord: Option<&Coordinates> = match code {
            DCode::Operation(Operation::Move(c)) => c.as_ref(),
            DCode::Operation(Operation::Interpolate(c, _)) => c.as_ref(),
            DCode::Operation(Operation::Flash(Some(c))) => Some(c),
            _ => None,
        };

        if let Some(c) = coord {
            pos.x = c.x.unwrap().into();
            pos.y = c.y.unwrap().into();

            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
        }
    }

    [min, max]
}

impl Alignment {
    pub const ALL: [Self; 5] = [
        Self::TopLeft,
        Self::TopRight,
        Self::BottomLeft,
        Self::BottomRight,
        Self::Center,
    ];

    pub fn name(&self) -> &str {
        match self {
            Alignment::TopLeft => "Top Left",
            Alignment::TopRight => "Top Right",
            Alignment::BottomLeft => "Bottom Left",
            Alignment::BottomRight => "Bottom Right",
            Alignment::Center => "Center",
        }
    }
}

impl Default for PrintedCircuitBoard {
    fn default() -> Self {
        Self {
            gerber: Default::default(),
            alignment: Default::default(),
            exposure_time: Minutes::new(3.0).convert(),
            invert: Default::default(),
        }
    }
}
