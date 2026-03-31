use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU64, Ordering},
};

use common::{
    slice::{EncodableLayer, SliceResult},
    units::Milimeter,
};
use itertools::Itertools;
use nalgebra::{Vector2, Vector3};
use ordered_float::OrderedFloat;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    geometry::Segments1D,
    slicer::{SEGMENT_LAYERS, Slicer},
};

impl Slicer {
    /// Actually runs the slicing operation, it is multithreaded.
    pub fn slice_raster<Layer: EncodableLayer>(&self) -> SliceResult<'_, Layer::Output> {
        let platform_resolution = self.slice_config.platform_resolution;
        let pixels = (platform_resolution.x * platform_resolution.y) as u64;
        let voxels = AtomicU64::new(0);

        // A segment contains a reference to all of the triangles it contains. By
        // splitting the mesh into segments, not all triangles need to be tested
        // to find all intersections. This massively speeds up the slicing
        // operation and actually makes it faster than most other slicers. :p
        let segments = (self.models.iter())
            .map(|model| Segments1D::from_mesh(&model.mesh, SEGMENT_LAYERS))
            .collect::<Vec<_>>();

        let layers = (0..self.layers)
            .into_par_iter()
            .map(|layer| {
                let height = layer as f32 * self.slice_config.slice_height.get::<Milimeter>();
                let mut voxels_inner = 0;

                // Gets all the intersections between the slice plane and the
                // model. Because all the faces are triangles, every triangle
                // intersection will return two points. These can then be
                // interpreted as line segments making up a polygon.
                let segments = (self.models.iter().enumerate()).flat_map(|(idx, model)| {
                    let intersections = segments[idx].intersect_plane(&model.mesh, height);
                    (intersections.into_iter()).map(|segment| (segment, model.exposure))
                });
                let mut edges = global_edge_table(segments);

                // Creates a new encoded for this layer. Because printers can
                // have very high resolution displays, the uncompressed data for
                // a sliced model can easily be over 30 Gigabytes. Most formats
                // use some sort of compression scheme to resolve this issue, so
                // to use a little memory as needed, the layers are compressed
                // as they are made.
                let mut encoder = Layer::new(self.slice_config.platform_resolution);
                let mut last = 0;

                let mut active = Vec::<ActiveEdge>::new();
                let mut y = edges.front().map(|e| e.min.y).unwrap_or(0);

                while !edges.is_empty() || !active.is_empty() {
                    update_active_edges(&mut edges, &mut active, y);
                    let y_offset = (platform_resolution.x * y) as u64;

                    // Convert the intersections into runs of voxels to be
                    // encoded into the layer.
                    let mut depth = 0;
                    let mut exposures = [0; 256];
                    for (a, b) in active.iter().tuple_windows() {
                        let delta = 1 - (a.direction as i32) * 2;
                        depth += delta;
                        exposures[a.exposure as usize] += delta;

                        let (a, b) = (a.x.round() as u64, b.x.round() as u64);
                        if depth != 0 && b != a {
                            let (start, length) = (a + y_offset, b - a);
                            (start > last).then(|| encoder.add_run(start - last, 0));

                            let exposure = exposures.iter().rposition(|&x| x > 0).unwrap_or(255);
                            encoder.add_run(length, exposure as u8);

                            voxels_inner += length;
                            last = start + length;
                        }
                    }

                    y += 1;
                }

                // Turns out that on my printer, the buffer that each layer is
                // decoded into is just uninitialized memory. So if the last run
                // doesn't fill the buffer, the printer will just print whatever
                // was in the buffer before which just makes a huge mess.
                (last < pixels).then(|| encoder.add_run(pixels - last, 0));

                // Finished encoding the layer
                voxels.fetch_add(voxels_inner, Ordering::Relaxed);
                encoder.finish(layer, &self.slice_config)
            })
            .inspect(|_| self.progress.add_complete(1))
            .collect::<Vec<_>>();

        self.progress.set_finished();
        SliceResult {
            layers,
            voxels: voxels.load(Ordering::Relaxed),
            slice_config: &self.slice_config,
        }
    }
}

#[derive(Debug)]
struct Edge {
    min: Vector2<u32>,

    y_max: u32,
    inv_slope: f32,
    direction: bool,
    exposure: u8,
}

#[derive(Debug)]
struct ActiveEdge {
    x: f32,

    y_max: u32,
    inv_slope: f32,
    direction: bool,
    exposure: u8,
}

fn global_edge_table(
    segments: impl Iterator<Item = (([Vector3<f32>; 2], bool), u8)>,
) -> VecDeque<Edge> {
    let mut edges = Vec::new();
    for ((pos, direction), exposure) in segments {
        let pos = pos.map(|x| x.map(|x| x.round() as u32));

        let dy = pos[1].y as f32 - pos[0].y as f32;
        let dx = pos[1].x as f32 - pos[0].x as f32;
        if dy == 0.0 {
            continue;
        }

        edges.push(Edge {
            min: pos[(pos[0].y >= pos[1].y) as usize].xy(),
            y_max: pos[0].y.max(pos[1].y),
            inv_slope: dx / dy,
            direction,
            exposure,
        });
    }

    edges.sort_by(|a, b| a.min.y.cmp(&b.min.y).then_with(|| a.min.x.cmp(&b.min.x)));
    VecDeque::from(edges)
}

fn update_active_edges(edges: &mut VecDeque<Edge>, active: &mut Vec<ActiveEdge>, y: u32) {
    active.retain(|x| x.y_max > y);
    active.iter_mut().for_each(|e| e.x += e.inv_slope);
    while !edges.is_empty() && edges[0].min.y == y {
        let edge = edges.pop_front().unwrap();
        active.push(ActiveEdge {
            x: edge.min.x as f32,
            y_max: edge.y_max,
            inv_slope: edge.inv_slope,
            direction: edge.direction,
            exposure: edge.exposure,
        });
    }
    active.sort_by_key(|x| OrderedFloat(x.x));
}
