use std::collections::VecDeque;

use common::{
    container::{Image, Run},
    progress::Progress,
    slice::DynSlicedFile,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Downsample {
    pub enabled: bool,
    pub factor: u8,
}

impl Downsample {
    pub fn post_slice(&self, file: &mut DynSlicedFile, progress: Progress) {
        if !self.enabled {
            return;
        }

        let layers = file.info().layers as u64;
        progress.set_total(layers);

        for i in 0..layers as usize {
            let layer = file.runs(i).collect::<VecDeque<_>>();
            let runs = downsample_adjacent(2, layer);
            let image = Image::from_decoder(file.info().resolution.cast(), runs.into_iter());
            file.overwrite_layer(i, image);
            progress.add_complete(1);
        }

        progress.set_finished();
    }
}

fn downsample_adjacent(factor: u8, mut runs: VecDeque<Run>) -> Vec<Run> {
    let factor = factor as u64;
    let mut out = Vec::new();

    let mut i = 0;
    while !runs.is_empty() {
        let run = runs.pop_front().unwrap();
        i += run.length;

        let mut remaining = i % factor;
        out.push(Run {
            length: run.length / factor,
            value: run.value,
        });

        // if not a clean split, we will need to do some averaging
        let mut interp = remaining * run.value as u64;
        while remaining > 0 {
            let length = factor - remaining;

            // try to complete remaining by pulling from next run
            let next = runs.front_mut().unwrap();
            interp += length * next.value as u64;
            next.length = next.length.saturating_sub(length);
            (next.length == 0).then(|| runs.pop_front());

            remaining += length;
            i += length;

            if remaining == factor {
                out.push(Run {
                    length: 1,
                    value: (interp / factor) as u8,
                });
                break;
            }
        }
    }

    out
}

// |--|--|--|--|
//  aa aa ab bb

//  a×5, b×3
// 5 % 2 = 1

#[test]
fn test_downsample_row() {
    let input = vec![
        Run {
            length: 5,
            value: 255,
        },
        Run {
            length: 3,
            value: 0,
        },
    ];
    let out = downsample_adjacent(2, input.into());
    dbg!(out);
}
