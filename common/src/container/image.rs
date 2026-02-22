use nalgebra::Vector2;

use super::Run;

/// Fast grayscale image buffer
#[derive(Clone)]
pub struct Image {
    pub size: Vector2<usize>,
    data: Vec<u8>,
    idx: usize,
}

/// Iterator of runs over a sequence of items.
pub struct ImageRuns<'a, T = u8> {
    inner: &'a [T],

    last_value: T,
    last_idx: u64,
}

impl Image {
    pub fn blank(width: usize, height: usize) -> Self {
        Self {
            size: Vector2::new(width, height),
            data: vec![0; width * height],
            idx: 0,
        }
    }

    pub fn from_decoder(size: Vector2<usize>, decoder: impl Iterator<Item = Run>) -> Self {
        let mut image = Self::blank(size.x, size.y);
        decoder.for_each(|run| image.add_run(run.length as usize, run.value));
        image
    }

    pub fn from_raw(size: Vector2<usize>, data: Vec<u8>) -> Self {
        Self { size, data, idx: 0 }
    }

    pub fn add_run(&mut self, length: usize, value: u8) {
        self.data[self.idx..self.idx + length].fill(value);
        self.idx += length;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let idx = self.size.x * y + x;
        self.data[idx]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        let idx = self.size.x * y + x;
        self.data[idx] = val;
    }

    pub fn runs(&self) -> ImageRuns<'_> {
        ImageRuns::new(&self.data)
    }

    pub fn inner_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn inner(&self) -> &[u8] {
        &self.data
    }

    pub fn take(self) -> Vec<u8> {
        self.data
    }
}

impl Image {
    pub fn blur(&mut self, sigma: f32) {
        let sigma = sigma as usize;
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let mut sum = 0;
                for xp in x.saturating_sub(sigma)..(x + sigma).min(self.size.x) {
                    for yp in y.saturating_sub(sigma)..(y + sigma).min(self.size.y) {
                        sum += self.get_pixel(xp, yp);
                    }
                }

                let avg = sum as f32 / (sigma * sigma) as f32;
                self.set_pixel(x, y, avg as u8);
            }
        }
    }
}

impl<'a, T: Default> ImageRuns<'a, T> {
    pub fn new(data: &'a [T]) -> Self {
        Self {
            inner: data,
            last_value: T::default(),
            last_idx: 0,
        }
    }
}

impl<T: PartialEq + Copy + Default> Iterator for ImageRuns<'_, T> {
    type Item = Run<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let size = self.inner.len() as u64;
        if self.last_idx >= size {
            return None;
        }

        for i in 0.. {
            let idx = self.last_idx + i;

            if idx >= size || self.inner[idx as usize] != self.last_value {
                let out = Run {
                    length: i,
                    value: self.last_value,
                };
                self.last_value = self.inner.get(idx as usize).copied().unwrap_or_default();
                self.last_idx += i;
                return Some(out);
            }
        }

        unreachable!()
    }
}
