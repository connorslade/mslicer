use std::collections::VecDeque;

use common::{
    container::{Image, Run},
    progress::Progress,
    slice::DynSlicedFile,
};
use itertools::Itertools;
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

pub fn downsample_adjacent(factor: u8, mut runs: VecDeque<Run>) -> Vec<Run> {
    debug_assert!(runs.iter().map(|x| x.length).sum::<u64>() % factor as u64 == 0);

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
            // try to complete remaining by pulling from next run
            let next = runs.front_mut().unwrap();
            let length = (factor - remaining).min(next.length);

            interp += length * next.value as u64;
            next.length -= length;
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

pub fn downsample(mut chunks: Vec<VecDeque<Run>>) -> Vec<Run> {
    let mut out = Vec::new();

    while !chunks[0].is_empty() {
        let shortest = (chunks.iter())
            .position_min_by_key(|x| x.front().unwrap().length)
            .unwrap();

        let length = chunks[shortest].front().unwrap().length;
        let mut value = 0;

        for chunk in chunks.iter_mut() {
            let front = chunk.front_mut().unwrap();
            value += front.value as u64;

            if front.length - length == 0 {
                chunk.pop_front();
            } else {
                front.length -= length;
            }
        }

        out.push(Run::new(length, (value / chunks.len() as u64) as u8));
    }

    out
}

// 1111000011
// 0011111000
// ↓
// ½½11½½½0½½

// or

// |11111111|00|
// |111111|0000|
// ↓
// 111111½½00
//
// 1. match up runs
// |111111|11|00|
// |111111|00|00|

#[test]
fn test_downsample() {
    let input = vec![
        vec![Run::new(8, 255), Run::new(2, 0)].into(),
        vec![Run::new(6, 255), Run::new(4, 0)].into(),
    ];
    let out = downsample(input);
    dbg!(out);
}
