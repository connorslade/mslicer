pub mod png;

use std::mem;

#[derive(Debug, Clone, Copy)]
pub struct Run {
    pub length: u64,
    pub value: u8,
}

pub fn find_islands(prev: &[u64], layer: &[u64]) -> bool {
    let mut layer_pos = 0;

    for (layer_idx, layer_run) in layer.iter().enumerate() {
        // if current layer run is of non-zero values (requires support from
        // previous layer)
        if layer_idx % 2 == 1 {
            let layer_end = layer_pos + layer_run;
            let mut prev_pos = 0;

            for (prev_idx, prev_run) in prev.iter().enumerate() {
                if prev_idx % 2 == 1 && layer_pos < prev_pos + prev_run && prev_pos < layer_end {
                    return true; // found island
                }
                prev_pos += prev_run;
            }
        }

        layer_pos += layer_run;
    }

    false
}

/// Returns a list of lengths, starting with zero and alternating. So `[0, 23,
/// 7]` would mean the run starts with 23 non-zero bytes, then 7 zero bytes.
pub fn condense_nonzero_runs(runs: &[Run]) -> Vec<u64> {
    let mut out = Vec::new();

    let mut value = false;
    let mut length = 0;
    for run in runs {
        let this_value = run.value > 0;
        if this_value ^ value {
            out.push(mem::replace(&mut length, run.length));
            value = this_value;
        } else {
            length += run.length;
        }
    }

    (length > 0).then(|| out.push(length));
    out
}

pub struct RunChunks<'a> {
    runs: &'a [Run],
    width: u64,

    index: usize,
    offset: u64,
}

impl<'a> RunChunks<'a> {
    pub fn new(runs: &'a [Run], width: u32) -> Self {
        Self {
            runs,
            width: width as u64,

            index: 0,
            offset: 0,
        }
    }
}

impl<'a> Iterator for RunChunks<'a> {
    type Item = Vec<Run>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.runs.len() {
            return None;
        }

        let mut out = Vec::new();
        let mut length = 0;

        while length < self.width && self.index < self.runs.len() {
            let run = self.runs[self.index];
            let run_length = run.length - self.offset;
            let clamped_run_length = run_length.min(self.width - length);
            length += clamped_run_length;
            out.push(Run {
                length: clamped_run_length,
                value: run.value,
            });

            if clamped_run_length == run_length {
                self.index += 1;
                self.offset = 0;
            } else {
                self.offset += clamped_run_length;
            }
        }

        Some(out)
    }
}
