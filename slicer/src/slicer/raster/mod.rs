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
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{
    geometry::Segments1D,
    slicer::{
        SEGMENT_LAYERS, Slicer,
        raster::edge_table::{global_edge_table, update_active_edges},
    },
};

mod edge_table;

impl Slicer {
    /// Actually runs the slicing operation, it is multithreaded.
    pub fn slice_raster(&self) -> Vec<Layer> {
        let supersample = self.slice_config.supersample;
        let remap = self.slice_config.exposure_remap.table();

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
            .map(|i| {
                let height = i as f32 / supersample as f32
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

                layer(supersample, real_platform, segments)
            })
            .chunks(supersample as usize)
            .enumerate()
            .map(|(i, mut chunk)| {
                let mut data = if supersample > 1 {
                    downsample_to_vec(&chunk, pixels)
                } else {
                    chunk.pop().unwrap()
                };

                data.iter_mut()
                    .filter(|x| x.value > 0)
                    .for_each(|x| x.value = remap[x.value as usize]);

                let exposure = self.slice_config.exposure_config(i as u32).into_owned();
                Layer { data, exposure }
            })
            .inspect(|_| self.progress.add_complete(1))
            .collect::<Vec<_>>()
    }
}

pub fn layer(
    supersample: u8,
    real_platform: Vector2<u32>,
    segments: impl Iterator<Item = (([Vector2<f32>; 2], bool), u8)>,
) -> Vec<Run> {
    let platform = real_platform * supersample as u32;

    let mut edges = global_edge_table(segments);
    let mut active = Vec::new();
    let first_y = edges.front().map(|e| e.min.y).unwrap_or(0);

    let mut runs = Vec::new();
    let padding = (first_y as u64 / supersample as u64) * real_platform.x as u64;
    runs.push(Run::new(padding, 0));

    let mut rows = vec![Vec::new(); supersample as usize];
    let mut row = Vec::new();
    let mut y = first_y;
    while (!edges.is_empty() || !active.is_empty()) && y < platform.y {
        update_active_edges(&mut edges, &mut active, y);
        let out = if supersample > 1 { &mut row } else { &mut runs };

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
                (start > last).then(|| out.push(Run::new(start - last, 0)));

                let exposure = exposures.iter().rposition(|&x| x > 0).unwrap_or(255);
                out.push(Run::new(length, exposure as u8));
                last = start + length;
            }
        }

        // Fill the empty space at the end of the row
        let padding = platform.x as u64 - last;
        (padding > 0).then(|| out.push(Run::new(padding, 0)));

        if supersample > 1 {
            let ss_row = ((y - first_y) % supersample as u32) as usize;
            downsample_adjacent(supersample, &row, &mut rows[ss_row]);
            row.clear();

            if (y + 1).is_multiple_of(supersample as u32) {
                downsample(&rows, platform.x as u64, &mut runs);
                rows.iter_mut().for_each(Vec::clear);
            }
        }

        y += 1;
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
}

fn downsample_to_vec(chunks: &[Vec<Run>], width: u64) -> Vec<Run> {
    let mut out = Vec::new();
    downsample(chunks, width, &mut out);
    out
}
