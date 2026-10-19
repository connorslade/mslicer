use std::{iter, mem, path::PathBuf, sync::Arc};

use common::{
    container::Run,
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeters,
};
use image::{GrayImage, imageops::FilterType};
use nalgebra::Vector2;

use crate::misc::alignment::Alignment;

#[derive(Default, Clone)]
pub struct Image {
    pub size: Sizing,
    pub alignment: Alignment,

    pub data: Option<Arc<(GrayImage, PathBuf)>>,
}

#[derive(Default, Clone)]
pub enum Sizing {
    #[default]
    Printer,
    Width(Milimeters),
    Height(Milimeters),
    Both {
        width: Milimeters,
        height: Milimeters,
    },
}

// todo: extend to support image sequences?
impl Image {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let image = &self.data.as_ref().unwrap().0;
        let platform = config.platform_resolution;
        let px = config.pixel_size();
        progress.set_total(1);

        let aspect = image.width() as f32 / image.height() as f32;
        let size = self.size.image_size(config.platform_size.xy(), aspect);
        let image_px = Vector2::new(size.x / px.x, size.y / px.y).map(|x| x.round() as u32);

        let scaled = image::imageops::resize(image, image_px.x, image_px.y, FilterType::Triangle);
        let padding = Padding::for_alignment(&self.alignment, platform.cast(), image_px.cast());

        // what even is this sob
        let runs = iter::once(Run::new(platform.x as u64 * padding.top, 0))
            .chain(image_runs(&scaled).into_iter().flat_map(|row| {
                iter::once(Run::new(padding.left, 0))
                    .chain(row)
                    .chain(iter::once(Run::new(padding.right, 0)))
            }))
            .chain(iter::once(Run::new(platform.x as u64 * padding.bottom, 0)))
            .filter(|x| x.length > 0)
            .collect();

        progress.set_finished();
        vec![Layer::new(
            runs,
            config.slice_height,
            config.exposure_config(0).into_owned(),
        )]
    }
}

impl Sizing {
    // aspect = width / height
    fn image_size(&self, printer: Vector2<Milimeters>, aspect: f32) -> Vector2<Milimeters> {
        match self {
            Sizing::Printer => printer,
            Sizing::Width(length) => Vector2::new(*length, *length / aspect),
            Sizing::Height(length) => Vector2::new(*length * aspect, *length),
            Sizing::Both { width, height } => Vector2::new(*width, *height),
        }
    }
}

fn image_runs(image: &GrayImage) -> Vec<Vec<Run>> {
    let size = Vector2::new(image.width(), image.height());
    let raw = image.as_raw();

    let mut rows = Vec::new();
    for y in 0..size.y as usize {
        let row = &raw[(y * size.x as usize)..((y + 1) * size.x as usize)];
        let mut runs = Vec::new();

        let mut active = Run::new(1, row[0]);
        for &value in &row[1..] {
            if active.value == value {
                active.length += 1;
            } else {
                runs.push(mem::replace(&mut active, Run::new(1, value)));
            }
        }

        runs.push(active);
        rows.push(runs);
    }

    rows
}

struct Padding {
    top: u64,
    left: u64,
    right: u64,
    bottom: u64,
}

impl Padding {
    fn for_alignment(align: &Alignment, total: Vector2<u64>, image: Vector2<u64>) -> Self {
        match align {
            Alignment::TopLeft => Self {
                top: 0,
                left: 0,
                right: total.x - image.x,
                bottom: total.y - image.y,
            },
            Alignment::TopCenter => todo!(),
            Alignment::TopRight => todo!(),
            Alignment::CenterLeft => todo!(),
            Alignment::Center => todo!(),
            Alignment::CenterRight => todo!(),
            Alignment::BottomLeft => todo!(),
            Alignment::BottomCenter => todo!(),
            Alignment::BottomRight => todo!(),
        }
    }
}
