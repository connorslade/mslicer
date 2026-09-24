use std::f32::consts::TAU;

use nalgebra::Vector3;
use slicer::builder::MeshBuilder;

use crate::misc::circle_points;

const QUAD: [u32; 4] = [0, 1, 2, 3];

pub fn add_disk(
    builder: &mut MeshBuilder,
    height: f32,
    outer_radius: f32,
    inner_radius: f32,
    sagitta: f32,
) -> [u32; 3] {
    let bo = builder.next_idx();
    let outer_points = circle_points(sagitta as f64, outer_radius as f64);
    for i in 0..outer_points {
        let point = disk_point(i as f32 / outer_points as f32) * outer_radius;
        builder.add_vertex(point);
        builder.add_vertex(point + Vector3::z() * height);
        builder.add_quad(QUAD.map(|x| bo + (i * 2 + x) % (outer_points * 2)));
    }

    let bi = builder.next_idx();
    let inner_points = circle_points(sagitta as f64, inner_radius as f64);
    for i in 0..inner_points {
        let point = disk_point(i as f32 / inner_points as f32) * inner_radius;
        builder.add_vertex(point);
        builder.add_vertex(point + Vector3::z() * height);

        builder.add_quad_flipped(QUAD.map(|x| bi + (i * 2 + x) % (inner_points * 2)));
    }

    let be = builder.next_idx();
    triangulate_gap(
        builder,
        VertexRing::new(bo, outer_points).with_step(2),
        VertexRing::new(bi, inner_points).with_step(2),
    );

    [bo, bi, be]
}

fn disk_point(t: f32) -> Vector3<f32> {
    let (y, x) = (t * TAU).sin_cos();
    Vector3::new(x, y, 0.0)
}

pub struct VertexRing {
    start: u32,
    len: u32,
    step: u32,
    offset: u32,
}

impl VertexRing {
    pub fn new(start: u32, len: u32) -> Self {
        Self {
            start,
            len,
            step: 1,
            offset: 0,
        }
    }

    pub fn with_step(self, step: u32) -> Self {
        Self { step, ..self }
    }

    pub fn with_offset(self, offset: u32) -> Self {
        Self { offset, ..self }
    }
}

pub fn triangulate_gap(builder: &mut MeshBuilder, a: VertexRing, b: VertexRing) {
    let (a, b) = if a.len < b.len { (b, a) } else { (a, b) };
    let a_per_b = a.len as f32 / b.len as f32;

    let mut remaining = 0.0;
    let mut ai = 0;
    for bi in 0..b.len {
        remaining += a_per_b;

        while remaining >= 1.0 {
            remaining -= 1.0;
            builder.add_face([
                b.start + (bi + b.offset) % b.len * b.step,
                a.start + (ai + a.offset + 1) % a.len * a.step,
                a.start + (ai + a.offset) % a.len * a.step,
            ]);
            ai += 1;
        }

        builder.add_face([
            b.start + (bi + b.offset) % b.len * b.step,
            b.start + (bi + b.offset + 1) % b.len * b.step,
            a.start + (ai + a.len) % a.len * a.step,
        ]);
    }

    builder.add_face([
        a.start + a.offset % a.len,
        b.start + (b.len + b.offset - 1) % b.len * b.step,
        b.start + b.offset % b.len,
    ]);
}
