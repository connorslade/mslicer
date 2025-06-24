use std::collections::{HashMap, HashSet};

use common::{
    misc::{VectorLayer, VectorSliceResult},
    serde::Serializer,
};
use nalgebra::Vector2;
use svg::{
    node::element::{
        tag::{Polygon, Polyline},
        Line, Polygon, Polyline, Rectangle,
    },
    Document,
};

pub struct SvgFile {
    layers: Vec<VectorLayer>,

    area: Vector2<u32>,
}

impl SvgFile {
    pub fn new(result: VectorSliceResult) -> Self {
        Self {
            layers: result.layers,

            area: result.slice_config.platform_resolution,
        }
    }

    pub fn layer_count(&self) -> u32 {
        self.layers.len() as u32
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        let sides = self.layer_count().isqrt() as usize;
        let (width, height) = (self.area.x as f32, self.area.y as f32);
        let mut svg = Document::new().set(
            "viewBox",
            (0, 0, width * sides as f32, height * (sides + 1) as f32),
        );

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

            let points = layer
                .points
                .iter()
                .map(|x| x.map(|x| (x * 1000.0).round() / 1000.0))
                .collect::<Vec<_>>();
            let polygons = join_segments(&points);
            for polygon in polygons {
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

fn join_segments(segments: &[Vector2<f32>]) -> Vec<Vec<Vector2<f32>>> {
    let mut polygons = Vec::new();

    fn point_to_key(point: &Vector2<f32>) -> (u32, u32) {
        (point.x.to_bits(), point.y.to_bits())
    }

    let mut points = HashMap::new();
    for segment in segments.chunks_exact(2) {
        points
            .entry(point_to_key(&segment[0]))
            .or_insert_with(Vec::new)
            .push(segment[1]);
        points
            .entry(point_to_key(&segment[1]))
            .or_insert_with(Vec::new)
            .push(segment[0]);
    }

    let mut seen = HashSet::new();
    while let Some(&start) = points.keys().find(|x| !seen.contains(*x)) {
        let mut polygon = Vec::new();
        let mut last = start;
        seen.insert(last);

        loop {
            let next_points = points.get(&last).unwrap();
            if let Some(next) = next_points
                .iter()
                .find(|x| !seen.contains(&point_to_key(x)))
            {
                polygon.push(*next);
                last = point_to_key(next);
                seen.insert(last);
            } else {
                break;
            }
        }

        if !polygon.is_empty() {
            polygons.push(polygon);
        }
    }

    polygons
}
