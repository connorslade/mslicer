use std::mem;

use common::{
    container::{
        Clusters,
        rle::{self, bits::ClusterRun},
    },
    progress::Progress,
    slice::Layer,
};
use nalgebra::Vector2;

pub fn detect_islands(
    resolution: Vector2<u32>,
    layers: &[Layer],
    progress: Progress,
    cascade: bool,
) -> Vec<Vec<u64>> {
    let [width, rows] = *resolution.cast::<u64>().as_ref();
    progress.set_total(layers.len() as u64);

    let mut prev = Vec::new();
    let mut curr = condensed_layer_rows(&layers[0], width);

    let mut annotations = Vec::new();
    for layer in layers.iter().skip(1) {
        // Convert the layer data to a mask of non-zero voxels, split by each row.
        progress.add_complete(1);
        mem::swap(&mut prev, &mut curr);
        curr = condensed_layer_rows(layer, width);

        // Group areas of adjacent pixels
        let mut clusters = Clusters::default();
        for row in 1..rows as usize {
            rle::bits::cluster_row_adjacency(&mut clusters, &curr, row - 1, row);
        }

        // Filter for clusters that are not supported by the previous layer
        let mut island_runs = Vec::<ClusterRun>::new();
        for (_, runs) in clusters.clusters() {
            // If a run on the layer below is adjacent to any run in this
            // cluster, it is considered supported. We can now check the next.
            if runs.iter().any(|run| row_overlaps(&prev, run)) {
                continue;
            }

            island_runs.extend(runs.iter());
        }

        let mut layer = Vec::new();
        let mut pos = 0;

        island_runs.sort_by(|a, b| a.row.cmp(&b.row).then(a.index.cmp(&b.index)));
        for run in island_runs.into_iter() {
            if cascade {
                curr[run.row][run.index - 1] += mem::take(&mut curr[run.row][run.index]);
            }

            let start = run.row as u64 * width + run.position;
            layer.push(start - pos);
            layer.push(run.size);
            pos = start + run.size;
        }

        annotations.push(layer);
    }

    progress.set_finished();
    annotations
}

fn condensed_layer_rows(layer: &Layer, width: u64) -> Vec<Vec<u64>> {
    let layer = rle::bits::from_runs(&layer.data);
    rle::bits::chunks(&layer, width)
}

fn row_overlaps(rows: &[Vec<u64>], run: &ClusterRun) -> bool {
    let mut prev_pos = 0;
    for (i, &x) in rows[run.row].iter().enumerate() {
        if i % 2 != 0
            && x > 0
            && run.position <= (prev_pos + x)
            && (run.position + run.size) >= prev_pos
        {
            return true;
        }

        prev_pos += x;
    }

    false
}
