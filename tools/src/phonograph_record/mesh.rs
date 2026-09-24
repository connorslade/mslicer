use std::{f32::consts::TAU, mem};

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
) {
    // todo
    // - faces around cylinders
    // faces to groove

    let bo = builder.next_idx();
    let outer_points = circle_points(sagitta as f64, outer_radius as f64);
    for i in 0..outer_points {
        let point = disk_point(i as f32 / outer_points as f32) * outer_radius;
        builder.add_vertex(point);
        builder.add_vertex(point + Vector3::z() * height);
        builder.add_quad_flipped(QUAD.map(|x| bo + (i * 2 + x) % (outer_points * 2)));
    }

    let bi = builder.next_idx();
    let inner_points = circle_points(sagitta as f64, inner_radius as f64);
    for i in 0..inner_points {
        let point = disk_point(i as f32 / inner_points as f32) * inner_radius;
        builder.add_vertex(point);
        builder.add_vertex(point + Vector3::z() * height);

        builder.add_quad(QUAD.map(|x| bi + (i * 2 + x) % (inner_points * 2)));
    }

    let be = builder.next_idx();
    triangulate_gap(builder, (bo, (bi - bo) / 2), (bi, (be - bi) / 2));
}

fn disk_point(t: f32) -> Vector3<f32> {
    let (x, y) = (t * TAU).sin_cos();
    Vector3::new(x, y, 0.0)
}

fn triangulate_gap(
    builder: &mut MeshBuilder,
    (mut a0, mut an): (u32, u32),
    (mut b0, mut bn): (u32, u32),
) {
    if an < bn {
        mem::swap(&mut a0, &mut b0);
        mem::swap(&mut an, &mut bn);
    }

    let a_per_b = an as f32 / bn as f32;

    let mut remaining = 0.0;
    let mut a = 0;
    for b in 0..bn {
        remaining += a_per_b;

        while remaining >= 1.0 {
            remaining -= 1.0;
            builder.add_face([b0 + b * 2, a0 + a * 2, a0 + (a + 1) % an * 2]);
            a += 1;
        }

        builder.add_face([b0 + b * 2, a0 + a * 2, b0 + (b + 1) % bn * 2]);
    }

    builder.add_face([a0, b0, b0 + (bn - 1) * 2]);
}
