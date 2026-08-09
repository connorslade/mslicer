//! Implementation of <https://paulbourke.net/geometry/polygonise>.

use std::{collections::HashMap, mem};

use common::{
    container::{
        Run,
        rle::{
            decode_into,
            downsample::{chunks, downsample, downsample_adjacent},
        },
    },
    progress::Progress,
    slice::Layer,
};
use itertools::Itertools;
use nalgebra::{Vector2, Vector3};
use ordered_float::OrderedFloat;

use crate::post_process::mesh_convert::table::{EDGE_TABLE, TRIANGULATION_TABLE};

#[rustfmt::skip]
const EDGE_CONNECTIONS: [(usize, usize); 12] = [
    (0, 1), (1, 2), (2, 3), (3, 0),
    (4, 5), (5, 6), (6, 7), (7, 4),
    (0, 4), (1, 5), (2, 6), (3, 7)
];

const GRID_POINTS: [Vector3<u32>; 8] = [
    Vector3::new(0, 0, 0),
    Vector3::new(1, 0, 0),
    Vector3::new(1, 0, 1),
    Vector3::new(0, 0, 1),
    Vector3::new(0, 1, 0),
    Vector3::new(1, 1, 0),
    Vector3::new(1, 1, 1),
    Vector3::new(0, 1, 1),
];

fn pad_layer(
    runs: &[Run],
    size: Vector2<u32>,
    factor: u8,
    pad_value: u8,
) -> (Vec<Run>, Vector2<u32>) {
    let factor = factor as u32;
    let padded_x = size.x.div_ceil(factor) * factor;
    let padded_y = size.y.div_ceil(factor) * factor;

    let mut out = Vec::new();
    for mut row in chunks(runs, size.x as u64) {
        let row_len: u64 = row.iter().map(|r| r.length).sum();
        if row_len < padded_x as u64 {
            row.push(Run {
                length: padded_x as u64 - row_len,
                value: pad_value,
            });
        }
        out.extend(row);
    }

    if size.y < padded_y {
        out.push(Run {
            length: (padded_y - size.y) as u64 * padded_x as u64,
            value: pad_value,
        });
    }

    (out, Vector2::new(padded_x, padded_y))
}

fn decode(res: Vector2<u32>, factor: u8, layer: &Layer) -> Vec<u8> {
    let (data, size) = pad_layer(&layer.data, res, factor, 10);

    let mut out = Vec::new();
    downsample_adjacent(factor, &data, &mut out);

    let chunks = chunks(&out, size.x as u64 / factor as u64);
    let mut out = Vec::new();
    for y in chunks.chunks(factor as usize) {
        downsample(y, size.x as u64 / factor as u64, &mut out);
    }

    let mut pixels = vec![0; (size.x / factor as u32 * size.y / factor as u32) as usize];
    decode_into(out, &mut pixels);
    pixels
}

// todo: consider non-uniform layer heights
pub fn marching_cubes(
    progress: &Progress,
    iso_level: f32,
    size: Vector2<u32>,
    layers: &[Layer],
    subsample: u8,
) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>) {
    progress.set_total(layers.len() as _);

    let mut vertex_lookup = HashMap::<Vector3<OrderedFloat<f32>>, u32>::new();
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    let subsample_size = Vector2::new(
        size.x.div_ceil(subsample as u32),
        size.y.div_ceil(subsample as u32),
    );

    let mut layer_this = Vec::new();
    let mut layer_next = decode(size, subsample, &layers[0]);

    for z in 0..layers.len() as u32 - 1 {
        mem::swap(&mut layer_this, &mut layer_next);
        layer_next = decode(size, subsample, &layers[z as usize + 1]);

        for (x, y) in (0..subsample_size.x - 1).cartesian_product(0..subsample_size.y - 1) {
            let mut grid = [(Vector3::zeros(), 0.0); 8];
            let mut cube_index = 0;

            for (i, offset) in GRID_POINTS.iter().enumerate() {
                let pos = Vector3::new(x, y, z) + offset;
                let index = pos.y * subsample_size.x + pos.x;

                let value = if offset.z == 1 {
                    &layer_next
                } else {
                    &layer_this
                }[index as usize] as f32
                    / 255.0;

                grid[i] = (
                    Vector3::new(pos.x * subsample as u32, pos.y * subsample as u32, pos.z)
                        .map(|x| x as f32),
                    value,
                );
                cube_index |= ((value < iso_level) as usize) << i;
            }

            let edge = EDGE_TABLE[cube_index];
            let mut vertlist = [Vector3::zeros(); 12];
            for (i, &(p1, p2)) in EDGE_CONNECTIONS
                .iter()
                .enumerate()
                .filter(|(i, _)| edge & (1 << i) != 0)
            {
                vertlist[i] = vertex_interp(iso_level, grid[p1], grid[p2]);
            }

            let triangles = TRIANGULATION_TABLE[cube_index];
            for triangle in triangles.chunks(3) {
                let mut get_point_idx = |vert: u8| {
                    let point = vertlist[vert as usize];
                    let orderd = point.map(OrderedFloat);
                    if let Some(&idx) = vertex_lookup.get(&orderd) {
                        return idx;
                    }

                    let idx = vertices.len() as u32;
                    vertices.push(point);
                    vertex_lookup.insert(orderd, idx);
                    idx
                };

                faces.push([
                    get_point_idx(triangle[0]),
                    get_point_idx(triangle[2]),
                    get_point_idx(triangle[1]),
                ]);
            }
        }

        progress.add_complete(1);
    }

    progress.set_finished();
    (vertices, faces)
}

fn vertex_interp(
    isolevel: f32,
    (point_1, val_1): (Vector3<f32>, f32),
    (point_2, val_2): (Vector3<f32>, f32),
) -> Vector3<f32> {
    let mu = (isolevel - val_1) / (val_2 - val_1);
    Vector3::new(
        point_1.x + mu * (point_2.x - point_1.x),
        point_1.y + mu * (point_2.y - point_1.y),
        point_1.z + mu * (point_2.z - point_1.z),
    )
}
