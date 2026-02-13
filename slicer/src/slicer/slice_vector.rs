use std::collections::HashSet;

use common::{
    container::{Image, Run},
    progress::Progress,
    serde::{DynamicSerializer, Serializer},
    slice::{Format, SliceInfo, SlicedFile, VectorLayer, VectorSliceResult},
    units::Milimeter,
};
use nalgebra::{Vector2, Vector3};
use ordered_float::OrderedFloat;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use svg::{
    Document,
    node::element::{Polygon, Rectangle},
};

use crate::{
    geometry::Segments1D,
    slicer::{SEGMENT_LAYERS, Slicer},
};

pub struct SvgFile {
    layers: Vec<VectorLayer>,
    area: Vector2<u32>,
}

impl Slicer {
    pub fn slice_vector(&self) -> VectorSliceResult<'_> {
        let segments = self
            .models
            .iter()
            .map(|x| Segments1D::from_mesh(x, SEGMENT_LAYERS))
            .collect::<Vec<_>>();

        let layers = (0..self.layers)
            .into_par_iter()
            .inspect(|_| self.progress.add_complete(1))
            .map(|layer| {
                let height = layer as f32 * self.slice_config.slice_height.get::<Milimeter>();

                let segments = self
                    .models
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, mesh)| segments[idx].intersect_plane(mesh, height))
                    .flat_map(|x| x.0)
                    .map(|x| x.xy())
                    .collect::<Vec<_>>();

                join_segments(&segments)
            })
            .collect::<Vec<_>>();

        self.progress.set_finished();
        VectorSliceResult {
            layers,
            slice_config: &self.slice_config,
        }
    }
}

// this can be made more efficient with some kinda spacial partitioning system
// (it's currently like n²), but it's like whatever. its fast enough for what
// i'm doing.
fn join_segments(segments_raw: &[Vector2<f32>]) -> Vec<Vec<Vector2<f32>>> {
    const DISTANCE_CUTOFF: f32 = 0.5;

    let mut segments = HashSet::new();
    for segment in segments_raw.chunks_exact(2) {
        segments.insert((segment[0].map(OrderedFloat), segment[1].map(OrderedFloat)));
    }

    let mut polygons = Vec::new();
    while let Some(&start) = segments.iter().next() {
        let idx = polygons.len();
        polygons.push(Vec::new());

        let mut last = start.1;
        while let Some((x @ (a, b), [a_dist, b_dist])) = segments
            .iter()
            .map(|x @ (a, b)| (x, [a, b].map(|x| (last - x).map(|x| *x).magnitude())))
            .min_by_key(|(_, [a, b])| OrderedFloat(a.min(*b)))
        {
            let next = if a_dist < b_dist { b } else { a };
            if *next == start.1 || (a_dist > DISTANCE_CUTOFF && b_dist > DISTANCE_CUTOFF) {
                segments.remove(&{ *x });
                break;
            }

            polygons[idx].push(next.map(|x| *x));
            last = *next;
            segments.remove(&{ *x });
        }
    }

    polygons
}

impl SvgFile {
    pub fn new(result: VectorSliceResult) -> Self {
        Self {
            layers: result.layers,
            area: result.slice_config.platform_resolution,
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        let sides = self.layers.len().isqrt() + 1;
        let (width, height) = (self.area.x as f32, self.area.y as f32);
        let size = (width * sides as f32, height * sides as f32);

        let mut svg = Document::new()
            .set("viewBox", (0, 0, size.0, size.1))
            .set("width", format!("{}mm", size.0))
            .set("height", format!("{}mm", size.1));

        for (idx, layer) in self.layers.iter().enumerate() {
            let (x, y) = (idx % sides, idx / sides);
            let offset = Vector2::new(x as f32 * width, y as f32 * height);

            svg = svg.add(
                Rectangle::new()
                    .set("x", offset.x)
                    .set("y", offset.y)
                    .set("width", width)
                    .set("height", height)
                    .set("fill", "none")
                    .set("stroke", "gray")
                    .set("stroke-width", "0.1"),
            );

            for polygon in layer.iter() {
                let points = polygon
                    .iter()
                    .map(|x| x + offset)
                    .map(|x| (x.x, x.y))
                    .collect::<Vec<_>>();
                let poly = Polygon::new()
                    .set("points", points)
                    .set("fill", "none")
                    .set("stroke", "black")
                    .set("stroke-width", "0.1");
                svg = svg.add(poly);
            }
        }

        ser.write_bytes(svg.to_string().as_bytes());
    }
}

impl SlicedFile for SvgFile {
    fn serialize(&self, ser: &mut DynamicSerializer, progress: Progress) {
        self.serialize(ser);
        progress.set_total(1);
        progress.set_finished();
    }

    fn set_preview(&mut self, _preview: &image::RgbaImage) {}

    fn info(&self) -> SliceInfo {
        SliceInfo {
            layers: self.layers.len() as u32,
            resolution: Vector2::zeros(),
            size: Vector3::default(),
            bottom_layers: 0,
        }
    }

    fn format(&self) -> Format {
        Format::Svg
    }

    fn runs(&self, _layer: usize) -> Box<dyn Iterator<Item = Run> + '_> {
        unimplemented!()
    }

    fn overwrite_layer(&mut self, _layer: usize, _image: Image) {
        unimplemented!()
    }
}
