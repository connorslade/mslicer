use std::{borrow::Cow, iter, mem, path::PathBuf, sync::Arc};

use common::{
    container::Run,
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeters,
};
use image::{GrayImage, imageops};
use nalgebra::Vector2;

use crate::misc::alignment::Alignment;

#[derive(Default, Clone)]
pub struct Image {
    pub size: Sizing,
    pub alignment: Alignment,
    pub rotation: Rotation,
    pub flip: Flip,
    pub background: u8,

    pub data: Option<Arc<(GrayImage, PathBuf)>>,
}

#[derive(Default, Clone, PartialEq)]
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

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Rotation {
    #[default]
    North,
    East,
    South,
    West,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Flip {
    #[default]
    None,
    Horizontal,
    Vertical,
}

// todo: extend to support image sequences?
impl Image {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let mut image = (self.rotation)
            .rotate(&self.data.as_ref().unwrap().0)
            .into_owned();
        self.flip.flip(&mut image);

        let platform = config.platform_resolution;
        let px = config.pixel_size();
        progress.set_total(1);

        let aspect = image.width() as f32 / image.height() as f32;
        let size = self.size.image_size(config.platform_size.xy(), aspect);
        let image_px = Vector2::new(size.x / px.x, size.y / px.y).map(|x| x.round() as u32);
        let padding = Padding::for_alignment(&self.alignment, platform.cast(), image_px.cast());

        // what even is this sob
        let remap = config.exposure_remap.table();
        let bg = self.background;
        let runs = iter::once(Run::new(platform.x as u64 * padding.top, bg))
            .chain(image_runs(&image, image_px).into_iter().flat_map(|row| {
                let image = (row.into_iter()).map(|x| Run::new(x.length, remap[x.value as usize]));
                iter::once(Run::new(padding.left, bg))
                    .chain(image)
                    .chain(iter::once(Run::new(padding.right, bg)))
            }))
            .chain(iter::once(Run::new(platform.x as u64 * padding.bottom, bg)))
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

struct Padding {
    top: u64,
    left: u64,
    right: u64,
    bottom: u64,
}

impl Padding {
    fn for_alignment(align: &Alignment, total: Vector2<u64>, image: Vector2<u64>) -> Self {
        let free = total - image;
        let (left, top) = match align {
            Alignment::TopLeft => (0, 0),
            Alignment::TopCenter => (free.x / 2, 0),
            Alignment::TopRight => (free.x, 0),
            Alignment::CenterLeft => (0, free.y / 2),
            Alignment::Center => (free.x / 2, free.y / 2),
            Alignment::CenterRight => (free.x, free.y / 2),
            Alignment::BottomLeft => (0, free.y),
            Alignment::BottomCenter => (free.x / 2, free.y),
            Alignment::BottomRight => (free.x, free.y),
        };

        Self {
            top,
            left,
            right: free.x - left,
            bottom: free.y - top,
        }
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

    pub fn name(&self) -> &str {
        match self {
            Sizing::Printer => "Max",
            Sizing::Width(..) => "Width",
            Sizing::Height(..) => "Height",
            Sizing::Both { .. } => "Dimensions",
        }
    }
}

impl Rotation {
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    pub fn name(&self) -> &str {
        match self {
            Rotation::North => "None",
            Rotation::East => "90°",
            Rotation::South => "180°",
            Rotation::West => "270°",
        }
    }

    fn rotate<'a>(&self, image: &'a GrayImage) -> Cow<'a, GrayImage> {
        match self {
            Rotation::North => Cow::Borrowed(image),
            Rotation::East => Cow::Owned(imageops::rotate90(image)),
            Rotation::South => Cow::Owned(imageops::rotate180(image)),
            Rotation::West => Cow::Owned(imageops::rotate270(image)),
        }
    }
}

impl Flip {
    pub const ALL: [Self; 3] = [Self::None, Self::Horizontal, Self::Vertical];

    pub fn name(&self) -> &str {
        match self {
            Flip::None => "None",
            Flip::Horizontal => "Horizontal",
            Flip::Vertical => "Vertical",
        }
    }

    pub fn flip(&self, image: &mut GrayImage) {
        match self {
            Flip::None => {}
            Flip::Horizontal => imageops::flip_horizontal_in_place(image),
            Flip::Vertical => imageops::flip_vertical_in_place(image),
        }
    }
}

fn image_runs(image: &GrayImage, size: Vector2<u32>) -> Vec<Vec<Run>> {
    let img_size = Vector2::new(image.width(), image.height()).cast::<u64>();
    let size = size.cast::<u64>();
    let raw = image.as_raw();

    let mut rows = Vec::with_capacity(size.y as usize);
    for y in 0..size.y {
        let sy = (y * img_size.y / size.y) as usize;
        let row = &raw[sy * img_size.x as usize..(sy + 1) * img_size.x as usize];
        let sample = |x: u64| row[(x * img_size.x / size.x) as usize];

        let mut runs = Vec::new();
        let mut active = Run::new(1, sample(0));
        for x in 1..size.x {
            let value = sample(x);
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
