// Uses an Active Edge Table (AET) for faster filling.
// Reference: https://www.cs.rit.edu/~icss571/filling/how_to.html

use std::collections::VecDeque;

use common::{
    container::{
        Run,
        rle::downsample::{downsample, downsample_adjacent},
    },
    slice::Layer,
    units::Milimeter,
};
use itertools::Itertools;
use nalgebra::Vector2;
use ordered_float::OrderedFloat;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{
    geometry::Segments1D,
    slicer::{SEGMENT_LAYERS, Slicer},
};

impl Slicer {
    /// Actually runs the slicing operation, it is multithreaded.
    pub fn slice_raster(&self) -> Vec<Layer> {
        let supersample = self.slice_config.supersample;
        let real_platform = self.slice_config.platform_resolution;
        let platform = real_platform * supersample as u32;
        let pixels = platform.x as u64 * platform.y as u64;

        // A segment contains a reference to all of the triangles it contains. By
        // splitting the mesh into segments, not all triangles need to be tested
        // to find all intersections. This massively speeds up the slicing
        // operation and actually makes it faster than most other slicers. :p
        let segments = (self.models.iter())
            .map(|model| Segments1D::from_mesh(&model.mesh, SEGMENT_LAYERS))
            .collect::<Vec<_>>();

        (0..self.layers * supersample as u32)
            .into_par_iter()
            .map(|layer| {
                let height = layer as f32 / supersample as f32
                    * self.slice_config.slice_height.get::<Milimeter>();

                // Gets all the intersections between the slice plane and the
                // model. Because all the faces are triangles, every triangle
                // intersection will return two points. These can then be
                // interpreted as line segments making up a polygon.
                let segments = self.models.iter().enumerate().flat_map(|(idx, model)| {
                    let intersections = segments[idx].intersect_plane(&model.mesh, height);
                    intersections.into_iter().map(|(pos, dir)| {
                        ((pos.map(|x| x * supersample as f32), dir), model.exposure)
                    })
                });
                let mut edges = global_edge_table(segments);

                let mut runs = Vec::new();

                let mut active = Vec::new();
                let first_y = edges.front().map(|e| e.min.y).unwrap_or(0);

                runs.push(Run::new(
                    (first_y as u64 / supersample as u64) * real_platform.x as u64,
                    0,
                ));

                let mut y = first_y;

                let mut rows = vec![Vec::new(); supersample as usize];
                let mut row = Vec::new();
                while (!edges.is_empty() || !active.is_empty()) && y < platform.y {
                    update_active_edges(&mut edges, &mut active, y);

                    // Convert the intersections into runs of voxels to be
                    // encoded into the layer.
                    let mut depth = 0;
                    let mut exposures = [0; 256];
                    let mut last = 0;
                    for (a, b) in active.iter().tuple_windows() {
                        let delta = 1 - (a.direction as i32) * 2;
                        depth += delta;
                        exposures[a.exposure as usize] += delta;

                        let [a, b] = [a.x, b.x].map(|x| (x.round() as u64).min(platform.x as u64));
                        if depth != 0 && b != a {
                            let (start, length) = (a, b - a);
                            (start > last).then(|| row.push(Run::new(start - last, 0)));

                            let exposure = exposures.iter().rposition(|&x| x > 0).unwrap_or(255);
                            row.push(Run::new(length, exposure as u8));
                            last = start + length;
                        }
                    }

                    // Fill the empty space at the end of the row
                    let padding = platform.x as u64 - last;
                    (padding > 0).then(|| row.push(Run::new(padding, 0)));

                    let ss_row = ((y - first_y) % supersample as u32) as usize;
                    downsample_adjacent(supersample, &row, &mut rows[ss_row]);
                    row.clear();

                    y += 1;
                    if y % supersample as u32 == 0 {
                        downsample(&rows, platform.x as u64, &mut runs);
                        rows.iter_mut().for_each(Vec::clear);
                    }
                }

                // Turns out that on my printer, the buffer that each layer is
                // decoded into is just uninitialized memory. So if the last run
                // doesn't fill the buffer, the printer will just print whatever
                // was in the buffer before which just makes a huge mess.
                if y < platform.y {
                    let rows = (real_platform.y - y / supersample as u32) as u64;
                    runs.push(Run::new(rows * real_platform.x as u64, 0));
                }

                runs
            })
            .chunks(supersample as usize)
            .enumerate()
            .map(|(i, chunk)| {
                let mut data = Vec::new();
                downsample(&chunk, pixels, &mut data);

                let exposure = self.slice_config.exposure_config(i as u32).clone();
                Layer { data, exposure }
            })
            .inspect(|_| self.progress.add_complete(1))
            .collect::<Vec<_>>()
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
    segments: impl Iterator<Item = (([Vector2<f32>; 2], bool), u8)>,
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
            min: pos[(pos[0].y >= pos[1].y) as usize],
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
