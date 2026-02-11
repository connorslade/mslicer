use std::mem;

use common::{
    container::{Clusters, rle},
    progress::Progress,
};

use crate::format::FormatSliceFile;

pub fn detect_islands(file: &FormatSliceFile, progress: Progress) {
    let info = file.info();
    let rows = info.resolution.y;
    progress.set_total(info.layers as u64);

    let mut prev = Vec::new();
    let mut curr = condensed_layer_rows(file, 0);

    for layer in 1..info.layers as usize {
        progress.set_complete(layer as u64);
        mem::swap(&mut prev, &mut curr);
        curr = condensed_layer_rows(file, layer);

        let mut clusters = Clusters::default();
        for row in 1..rows as usize {
            rle::bits::cluster_row_adjacency(&mut clusters, &curr, row - 1, row);
        }

        println!("layer #{layer} has {} cluster(s)", clusters.cluster_count());

        // todo: island detection logic
        // - find clusters in curr
        // - filter clusters that are supported by prev
        // - generate a RLE annotation list
    }

    progress.set_finished();
}

fn condensed_layer_rows(file: &FormatSliceFile, layer: usize) -> Vec<Vec<u64>> {
    let layer = rle::bits::from_runs(&file.runs(layer).collect::<Vec<_>>());
    let size = file.info().resolution;
    rle::bits::chunks(&layer, size.x as u64)
}
