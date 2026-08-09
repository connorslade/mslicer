//! Implementation of <https://paulbourke.net/geometry/polygonise>.

use std::{collections::HashMap, mem};

use common::{container::rle::decode_into, progress::Progress, slice::Layer};
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

// todo: consider non-uniform layer heights
pub fn marching_cubes(
    progress: &Progress,
    iso_level: f32,
    size: Vector2<u32>,
    layers: &[Layer],
) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>) {
    progress.set_total(layers.len() as _);

    let mut vertex_lookup = HashMap::<Vector3<OrderedFloat<f32>>, u32>::new();
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    let pixels = (size.x * size.y) as usize;
    let mut layer_this = vec![0; pixels];
    let mut layer_next = vec![0; pixels];
    decode_into(&layers[0].data, &mut layer_next);

    for z in 0..layers.len() as u32 - 1 {
        mem::swap(&mut layer_this, &mut layer_next);
        decode_into(&layers[z as usize + 1].data, &mut layer_next);

        for (x, y) in (0..size.x - 1).cartesian_product(0..size.y - 1) {
            let mut grid = [(Vector3::zeros(), 0.0); 8];
            let mut cube_index = 0;

            for (i, offset) in GRID_POINTS.iter().enumerate() {
                let pos = Vector3::new(x, y, z) + offset;

                let index = pos.x * size.y + pos.y;
                let value = if offset.z == 1 {
                    &layer_next
                } else {
                    &layer_this
                }[index as usize] as f32
                    / 255.0;

                grid[i] = (pos.map(|x| x as f32), value);
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
                    get_point_idx(triangle[1]),
                    get_point_idx(triangle[2]),
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
