use std::time::Duration;

use nalgebra::Vector2;
use rand::{distr::Alphanumeric, Rng};

use crate::config::SliceConfig;

pub struct SliceResult<'a, Layer> {
    pub layers: Vec<Layer>,
    pub slice_config: &'a SliceConfig,
}

pub struct VectorSliceResult<'a> {
    pub layers: Vec<VectorLayer>,
    pub slice_config: &'a SliceConfig,
}

pub struct VectorLayer {
    pub polygons: Vec<Vec<Vector2<f32>>>,
}

#[derive(Debug)]
pub struct Run {
    pub length: u64,
    pub value: u8,
}

pub trait EncodableLayer: Default {
    type Output: Send;

    fn add_run(&mut self, length: u64, value: u8);
    fn finish(self, layer: u64, config: &SliceConfig) -> Self::Output;
}

pub fn human_duration(duration: Duration) -> String {
    let ms = duration.as_millis() as f32;
    if ms < 1000.0 {
        format!("{ms}ms")
    } else if ms < 60_000.0 {
        format!("{:.2}s", ms / 1000.0)
    } else if ms < 3_600_000.0 {
        let minutes = ms / 60_000.0;
        let seconds = (minutes - minutes.floor()) * 60.0;
        format!("{:.0}m {:.2}s", minutes.floor(), seconds)
    } else {
        let hours = ms / 3_600_000.0;
        let minutes = (hours - hours.floor()) * 60.0;
        let seconds = (minutes - minutes.floor()) * 60.0;
        format!(
            "{:.0}h {:.0}m {:.2}s",
            hours.floor(),
            minutes.floor(),
            seconds
        )
    }
}

pub fn random_string(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn subscript_number(num: impl Into<u64>) -> String {
    const SUBSCRIPT: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

    let mut num = num.into();
    let mut out = String::new();
    while num > 0 {
        out.push(SUBSCRIPT[(num % 10) as usize]);
        num /= 10;
    }

    out
}

pub const fn as_array<T, const N: usize>(this: &[T]) -> Option<&[T; N]> {
    if this.len() == N {
        let ptr = this.as_ptr() as *const [T; N];
        Some(unsafe { &*ptr })
    } else {
        None
    }
}
