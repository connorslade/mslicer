use std::path::PathBuf;

use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use glam::dvec3;
use itertools::Itertools;
use nalgebra::Vector2;
use opencascade::{
    bounding_box::aabb,
    primitives::{IntoShape, Shape},
    section,
    workplane::Workplane,
};
use slicer::slicer::raster::{Segment, layer};

#[derive(Default, Clone)]
pub struct BrepSlicer {
    pub step: Option<PathBuf>,
}

impl BrepSlicer {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let shape = Shape::read_step(self.step.as_ref().unwrap()).unwrap();

        let bounds = aabb(&shape);
        let (min, max) = (bounds.min(), bounds.max());
        let size = max - min;

        let platform = config.platform_resolution;
        let slice_height = config.slice_height.get::<Milimeter>();
        let layers = (size.z / slice_height as f64).round() as u32;
        progress.set_total(layers as u64);

        let center = (min + max) / 2.0;
        let width = (max.x - min.x) * 1.1 + 1.0;
        let depth = (max.y - min.y) * 1.1 + 1.0;

        (0..layers)
            .into_iter()
            .map(|i| {
                let height = (i as f64 + 0.5) * slice_height as f64;

                let plane = Workplane::xy()
                    .translated(dvec3(center.x, center.y, height))
                    .rect(width, depth)
                    .to_face()
                    .into_shape();

                let mut segments = Vec::new();
                for edge in section::edges(&shape, &plane).iter().flat_map(Shape::edges) {
                    for (a, b) in edge.approximation_segments().tuple_windows() {
                        segments.push(Segment {
                            endpoints: [a, b].map(|x| Vector2::new(x.x, x.y).cast::<f32>()),
                            entering: (a.y - b.y) > 0.0, // idk
                            priority: 255,
                            exposure: 255,
                        });
                    }
                }

                let (data, defects) = layer(1, platform, segments.into_iter());
                Layer::new(
                    data,
                    config.default_height(i),
                    config.exposure_config(i).into_owned(),
                )
                .with_defects(defects)
            })
            .inspect(|_| progress.add_complete(1))
            .collect()
    }
}
