//! Tools for working with run length encoded (RLE) data.

use std::{borrow::Borrow, collections::VecDeque};

use itertools::Itertools;

pub mod bits;
pub mod png;

/// Sequence of identical items.
#[derive(Debug, Clone, Copy)]
pub struct Run<T = u8> {
    pub length: u64,
    pub value: T,
}

impl<T> Run<T> {
    pub fn new(length: u64, value: T) -> Self {
        Self { length, value }
    }
}

/// Decode a RLE sequence into a mutable slice.
pub fn decode_into<T, R, D>(decoder: D, image: &mut [T])
where
    T: Clone,
    R: Borrow<Run<T>>,
    D: IntoIterator<Item = R>,
{
    let mut pixel = 0;
    for run in decoder {
        let run = run.borrow();
        let length = run.length as usize;
        image[pixel..(pixel + length)].fill(run.value.clone());
        pixel += length;
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
