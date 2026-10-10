use std::{path::PathBuf, sync::Arc};

use anyhow::{Ok, Result};
use common::progress::Progress;
use image::{ImageBuffer, Luma};
use nalgebra::{Vector2, Vector3};
use slicer::{builder::MeshBuilder, mesh::Mesh};

type Image = ImageBuffer<Luma<f32>, Vec<f32>>;

#[derive(Clone)]
pub struct HeightMap {
    pub size: Vector2<f32>,
    pub max_height: f32,

    pub data: Option<Arc<(Image, PathBuf)>>,
}

impl HeightMap {
    pub fn generate(&self, progress: &Progress) -> Result<Mesh> {
        let data = &self.data.as_ref().unwrap().0;
        let resolution = Vector2::new(data.width(), data.height());
        let offset = (-self.size / 2.0).to_homogeneous();

        progress.set_total(resolution.y as u64 * resolution.x as u64);
        let mut builder = MeshBuilder::new();
        for y in 0..resolution.y {
            let py = y as f32 / resolution.y as f32 * self.size.y;

            for x in 0..resolution.x {
                let px = x as f32 / resolution.x as f32 * self.size.x;

                let value = data.get_pixel(x, y);
                let height = value.0[0] * self.max_height;

                builder.add_vertex(Vector3::new(px, py, height) + offset);
                if x + 1 < resolution.x && y + 1 < resolution.y {
                    let row_this = resolution.x * y;
                    let row_next = resolution.x * (y + 1);

                    builder.add_quad_flipped([
                        row_this + x,
                        row_this + x + 1,
                        row_next + x,
                        row_next + x + 1,
                    ]);
                }

                progress.add_complete(1);
            }
        }

        progress.set_finished();
        Ok(builder.build())
    }
}

impl Default for HeightMap {
    fn default() -> Self {
        Self {
            size: Vector2::new(50.0, 50.0),
            max_height: 10.0,

            data: None,
        }
    }
}
