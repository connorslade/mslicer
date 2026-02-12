use std::mem;

use common::{
    container::{Clusters, Run, rle},
    progress::Progress,
};

use crate::format::FormatSliceFile;

pub fn detect_islands(file: &FormatSliceFile, progress: Progress) -> Vec<Vec<u64>> {
    let info = file.info();
    let [width, rows] = *info.resolution.as_ref();
    progress.set_total(info.layers as u64);

    let mut prev = Vec::new();
    let mut curr = condensed_layer_rows(file, 0);

    let mut annotations = Vec::new();
    for layer in 1..info.layers as usize {
        // Convert the layer data to a mask of non-zero voxels, split by each row.
        progress.set_complete(layer as u64);
        mem::swap(&mut prev, &mut curr);
        curr = condensed_layer_rows(file, layer);

        // Group areas of adjacent pixels
        let mut clusters = Clusters::default();
        for row in 1..rows as usize {
            rle::bits::cluster_row_adjacency(&mut clusters, &curr, row - 1, row);
        }

        // Filter for clusters that are not supported by the previous layer
        let mut island_runs = Vec::new();
        for (_, runs) in clusters.clusters() {
            // If a run on the layer below is adjacent to any run in this
            // cluster, it is considered supported. We can now check the next.
            if runs.iter().any(|run| row_overlaps(&prev, run)) {
                continue;
            }

            island_runs.extend(
                runs.iter()
                    .map(|(row, pos, size)| (*row as u64 * width as u64 + *pos, *size)),
            );
            println!("found island on layer #{layer}");
        }

        let mut layer = Vec::new();
        let mut pos = 0;

        island_runs.sort_by_key(|(start, _size)| *start);
        for (start, size) in island_runs.into_iter() {
            (layer.len() % 2 == 0).then(|| layer.push(start - pos)); // todo: will this always run?
            layer.push(size as u64);
            pos = start + size;
        }

        annotations.push(layer);
    }

    progress.set_finished();
    annotations
}

fn condensed_layer_rows(file: &FormatSliceFile, layer: usize) -> Vec<Vec<u64>> {
    let layer = rle::bits::from_runs(&file.runs(layer).collect::<Vec<_>>());
    let size = file.info().resolution;
    rle::bits::chunks(&layer, size.x as u64)
}

fn row_overlaps(rows: &[Vec<u64>], run: &(usize, u64, u64)) -> bool {
    let (row, pos, size) = (&rows[run.0], run.1, run.2);

    let mut prev_pos = 0;
    for (i, x) in row.iter().enumerate() {
        if i % 2 != 0 && pos <= (prev_pos + x) && (pos + size) >= prev_pos {
            return true;
        }

        prev_pos += x;
    }

    false
}
