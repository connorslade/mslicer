use std::{path::PathBuf, sync::Arc};

use anyhow::{Ok, Result};
use common::progress::Progress;
use image::{ImageBuffer, Luma};
use nalgebra::{Vector2, Vector3};
use slicer::geometry::mesh::{Mesh, MeshBuilder};

type Image = ImageBuffer<Luma<f32>, Vec<f32>>;

#[derive(Clone)]
pub struct HeightMap {
    pub size: Vector2<f32>,
    pub base_height: f32,
    pub max_height: f32,

    pub data: Option<Arc<(Image, PathBuf)>>,
}

impl HeightMap {
    pub fn generate(&self, progress: &Progress) -> Result<Mesh> {
        let data = &self.data.as_ref().unwrap().0;
        let resolution = Vector2::new(data.width(), data.height());
        let offset = (-self.size / 2.0).push(self.base_height);
        let position = |x: u32, y: u32| {
            Vector2::new(x, y)
                .cast::<f32>()
                .component_div(&(resolution - Vector2::repeat(1)).cast())
                .component_mul(&self.size)
        };

        progress.set_total(resolution.y as u64 * resolution.x as u64);
        let mut builder = MeshBuilder::new();
        for y in 0..resolution.y {
            for x in 0..resolution.x {
                let value = data.get_pixel(resolution.x - x - 1, y);
                let height = value.0[0] * self.max_height;

                builder.add_vertex(position(x, y).push(height) + offset);
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

        let (rx, ry) = (resolution.x, resolution.y);
        let perimeter = ((0..rx - 1).map(|x| (x, 0)))
            .chain((0..ry - 1).map(|y| (rx - 1, y)))
            .chain((1..rx).rev().map(|x| (x, ry - 1)))
            .chain((1..ry).rev().map(|y| (0, y)))
            .collect::<Vec<_>>();

        let center = builder.add_vertex(Vector3::zeros());
        let bottom = builder.next_idx();
        for &(x, y) in &perimeter {
            builder.add_vertex((position(x, y) + offset.xy()).to_homogeneous());
        }

        for i in 0..perimeter.len() {
            let j = (i + 1) % perimeter.len();
            let (top_i, top_j) = (perimeter[i], perimeter[j]);
            let (bot_i, bot_j) = (bottom + i as u32, bottom + j as u32);

            builder.add_quad_flipped([
                bot_i,
                bot_j,
                top_i.1 * rx + top_i.0,
                top_j.1 * rx + top_j.0,
            ]);
            builder.add_face([center, bot_j, bot_i]);
        }

        progress.set_finished();
        Ok(builder.build())
    }
}

impl Default for HeightMap {
    fn default() -> Self {
        Self {
            size: Vector2::new(50.0, 50.0),
            max_height: 5.0,
            base_height: 5.0,

            data: None,
        }
    }
}
