use std::path::PathBuf;

use common::{
    container::Run,
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use cxx::UniquePtr;
use glam::{DVec3, dvec3};
use itertools::Itertools;
use nalgebra::Vector2;
use opencascade::primitives::Edge;
use opencascade::{
    bounding_box::aabb,
    primitives::{IntoShape, Shape},
    section,
    workplane::Workplane,
};
use opencascade_sys::{
    self,
    b_rep::TopoDS_Edge,
    b_rep_adaptor::BRepAdaptor_Curve_new,
    gc_pnts::{GCPnts_TangentialDeflection_Value, TangentialDeflection_new},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use slicer::slicer::raster::{
    Segment,
    edge_table::{global_edge_table, update_active_edges},
};

#[derive(Default, Clone)]
pub struct BrepSlicer {
    pub step: Option<PathBuf>,
}

impl BrepSlicer {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let mut shape = Shape::read_step(self.step.as_ref().unwrap()).unwrap();

        let platform_size = (config.platform_size.xy()).map(|x| x.get::<Milimeter>());
        let platform = config.platform_resolution.cast::<f32>();
        let mm_to_px = platform.component_div(&platform_size).push(1.0);
        let offset = platform / 2.0;

        let bounds = aabb(&shape);
        let (min, max) = (bounds.min(), bounds.max());
        let size = max - min;
        shape.set_global_translation(dvec3(0.0, 0.0, -min.z));

        let platform = config.platform_resolution;
        let slice_height = config.slice_height.get::<Milimeter>();
        let layers = (size.z / slice_height as f64).round() as u32;
        progress.set_total(layers as u64);

        let center = (min + max) / 2.0;
        let width = (max.x - min.x) * 1.1 + 1.0;
        let depth = (max.y - min.y) * 1.1 + 1.0;

        // behold, the power of human creativity
        let ptr = &shape as *const Shape as usize;
        (0..layers)
            .into_par_iter()
            .map(|i| {
                let shape = unsafe { &*(ptr as *const Shape) };
                let height = (i as f64 + 0.5) * slice_height as f64;

                let plane = Workplane::xy()
                    .translated(dvec3(center.x, center.y, height))
                    .rect(width, depth)
                    .to_face()
                    .into_shape();

                let mut segments = Vec::new();
                for edge in section::edges(shape, &plane).iter().flat_map(Shape::edges) {
                    for (a, b) in approximate(&edge, 0.001, 0.001).tuple_windows() {
                        segments.push(Segment {
                            endpoints: [a, b].map(|x| {
                                Vector2::new(x.x, x.y)
                                    .cast::<f32>()
                                    .component_mul(&mm_to_px.xy())
                                    + offset
                            }),

                            // unused...
                            entering: false,
                            priority: 0,
                            exposure: 0,
                        });
                    }
                }

                let (data, defects) = layer_simple(platform, segments.into_iter());
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

fn layer_simple(
    platform: Vector2<u32>,
    segments: impl Iterator<Item = Segment>,
) -> (Vec<Run>, u64) {
    let mut edges = global_edge_table(segments);
    let mut active = Vec::new();
    let first_y = edges
        .front()
        .map(|e| (e.p_min.y - 0.5).ceil().max(0.0) as u32)
        .unwrap_or_default();
    let mut runs = Vec::new();
    let padding = first_y as u64 * platform.x as u64;

    (padding > 0).then(|| runs.push(Run::new(padding, 0)));

    let mut errors = 0;
    let mut y = first_y;

    while (!edges.is_empty() || !active.is_empty()) && y < platform.y {
        update_active_edges(&mut edges, &mut active, y);

        let mut depth = 0;
        let mut last = 0;
        for (i, (a, b)) in active.iter().tuple_windows().enumerate() {
            depth += [-1, 1][(i % 2 == 0) as usize];

            let [a, b] = [a.x, b.x].map(|x| {
                let pixel_x = (x - 0.5).ceil().max(0.0) as u64;
                pixel_x.min(platform.x as u64)
            });
            if depth != 0 && b != a {
                let (start, length) = (a, b - a);
                (start > last).then(|| runs.push(Run::new(start - last, 0)));

                runs.push(Run::new(length, 255));
                last = start + length;
            }
        }

        if let Some(last) = active.last() {
            errors += ((depth + 1 - (last.entering as i32) * 2) != 0) as u64;
        }

        let padding = platform.x as u64 - last;
        (padding > 0).then(|| runs.push(Run::new(padding, 0)));
        y += 1;
    }

    if y < platform.y {
        let rows = (platform.y - y) as u64;
        runs.push(Run::new(rows * platform.x as u64, 0));
    }

    (runs, errors)
}

fn edge_ffi(edge: &Edge) -> &TopoDS_Edge {
    unsafe { &*(edge as *const Edge as *const UniquePtr<TopoDS_Edge>) }
}

fn approximate(edge: &Edge, angular: f64, sag: f64) -> impl Iterator<Item = DVec3> {
    let curve = BRepAdaptor_Curve_new(edge_ffi(edge));
    let approx = TangentialDeflection_new(&curve, angular, sag);
    (1..=approx.NbPoints()).map(move |i| {
        let p = GCPnts_TangentialDeflection_Value(&approx, i);
        dvec3(p.X(), p.Y(), p.Z())
    })
}
