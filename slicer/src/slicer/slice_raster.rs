use std::sync::atomic::{AtomicU64, Ordering};

use common::{
    slice::{EncodableLayer, SliceResult},
    units::Milimeter,
};
use itertools::Itertools;
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
            .map(|x| Segments1D::from_mesh(x, SEGMENT_LAYERS))
            .collect::<Vec<_>>();

        let layers = (0..self.layers)
            .into_par_iter()
            .map(|layer| {
                let height = layer as f32 * self.slice_config.slice_height.get::<Milimeter>();

                // Gets all the intersections between the slice plane and the
                // model. Because all the faces are triangles, every triangle
                // intersection will return two points. These can then be
                // interpreted as line segments making up a polygon.
                let segments = (self.models.iter().enumerate())
                    .flat_map(|(idx, mesh)| {
                        let exposure = self.exposures[idx];
                        segments[idx]
                            .intersect_plane(mesh, height)
                            .into_iter()
                            .map(move |segment| (segment, exposure))
                    })
                    .collect::<Vec<_>>();

                // Creates a new encoded for this layer. Because printers can
                // have very high resolution displays, the uncompressed data for
                // a sliced model can easily be over 30 Gigabytes. Most formats
                // use some sort of compression scheme to resolve this issue, so
                // to use a little memory as needed, the layers are compressed
                // as they are made.
                let mut encoder = Layer::new(self.slice_config.platform_resolution);
                let mut last = 0;

                // For each row of pixels, we find all line segments that go
                // across and mark that as an intersection to then be run-length
                // encoded. There is probably a better polygon filling algo, but
                // this one works surprisingly fast.
                for y in 0..platform_resolution.y {
                    let y_offset = (platform_resolution.x * y) as u64;
                    let yf = y as f32 + 0.5;

                    let mut intersections = (segments.iter())
                        .map(|((pos, facing), exposure)| (pos[0], pos[1], facing, exposure))
                        // Filtering to only consider segments with one point
                        // above the current row and one point below.
                        .filter(|&(a, b, _, _)| (a.y >= yf) ^ (b.y >= yf))
                        .map(|(a, b, &facing, &exposure)| {
                            // Get the x position of the line segment at this y
                            let t = (yf - a.y) / (b.y - a.y);
                            let pos = a.x + t * (b.x - a.x);
                            (pos.round() as u64, facing, exposure)
                        })
                        .collect::<Vec<_>>();

                    // Sort all these intersections for run-length encoding
                    intersections.sort_by_key(|&(x, _, _)| x);

                    // Convert the intersections into runs of voxels to be
                    // encoded into the layer.
                    let mut depth = 0;
                    let mut exposures = [0; 256];
                    for ((a, a_dir, a_exposure), (b, _, _)) in
                        intersections.into_iter().tuple_windows()
                    {
                        let delta = 1 - (a_dir as i32) * 2;
                        depth += delta;
                        exposures[a_exposure as usize] += delta;

                        if depth != 0 && b != a {
                            let (start, length) = (a + y_offset, b - a);
                            (start > last).then(|| encoder.add_run(start - last, 0));

                            let max_exposure =
                                exposures.iter().rposition(|&x| x > 0).unwrap_or(255);
                            encoder.add_run(length, max_exposure as u8);

                            voxels.fetch_add(length, Ordering::Relaxed);
                            last = start + length;
                        }
                    }
                }

                // Turns out that on my printer, the buffer that each layer is
                // decoded into is just uninitialized memory. So if the last run
                // doesn't fill the buffer, the printer will just print whatever
                // was in the buffer before which just makes a huge mess.
                (last < pixels).then(|| encoder.add_run(pixels - last, 0));

                // Finished encoding the layer
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
