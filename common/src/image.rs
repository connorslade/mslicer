use nalgebra::Vector2;

use crate::misc::Run;

/// A fast grayscale image buffer
pub struct Image {
    size: Vector2<usize>,
    data: Vec<u8>,
    idx: usize,
}

impl Image {
    pub fn blank(width: usize, height: usize) -> Self {
        Self {
            size: Vector2::new(width, height),
            data: vec![0; width * height],
            idx: 0,
        }
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

    pub fn blur(&mut self, sigma: f32) {
        // Generate kernel
        let kernel_size = (4.0 * sigma) as usize;
        let half_kernel_size = kernel_size / 2;

        let mut kernel = vec![0.0; kernel_size];
        for (i, e) in kernel.iter_mut().enumerate().take(kernel_size) {
            *e = gaussian((i - half_kernel_size) as f32, sigma);
        }

        // Blur image horizontally
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let sum = (x.saturating_sub(half_kernel_size)
                    ..(x + half_kernel_size).min(self.size.x))
                    .map(|i| self.get_pixel(i, y) as f32 * kernel[i + half_kernel_size])
                    .sum::<f32>();
                self.set_pixel(x, y, (sum / kernel_size as f32) as u8);
            }
        }

        // Blur image vertically
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let sum = (y.saturating_sub(half_kernel_size)
                    ..(y + half_kernel_size).min(self.size.y))
                    .map(|i| self.get_pixel(i, y) as f32 * kernel[i + half_kernel_size])
                    .sum::<f32>();
                self.set_pixel(x, y, (sum / kernel_size as f32) as u8);
            }
        }
    }

    // TODO: Turn into iterator
    pub fn runs(&self) -> Vec<Run> {
        let mut last = (self.data[0], 0);
        let mut runs = Vec::new();

        let size = (self.size.x * self.size.y) as u64;
        for i in 0..size {
            let val = self.data[i as usize];

            if val != last.0 {
                runs.push(Run {
                    length: i - last.1,
                    value: last.0,
                });
                last = (val, i);
            }
        }

        if last.1 + 1 != size {
            runs.push(Run {
                length: size - last.1,
                value: 0,
            });
        }

        runs
    }

    pub fn finish(&self) -> &[u8] {
        &self.data
    }

    pub fn take(self) -> Vec<u8> {
        self.data
    }
}

fn gaussian(x: f32, sigma: f32) -> f32 {
    const ROOT_TWO_PI: f32 = 2.506_628_3;
    (x.powi(2) / (2.0 * sigma.powi(2))).exp() / (sigma * ROOT_TWO_PI)
}
