use common::geometry::convex_hull;
use nalgebra::Vector2;
use slicer::builder::MeshBuilder;

// todo: the config values are stored in support generator struct but not used
// through it...
pub fn build_raft(
    raft_offset: f32,
    raft_height: f32,
    points: &[Vector2<f32>],
    builder: &mut MeshBuilder,
) {
    let hull = convex_hull(points);
    let idx = builder.next_idx();
    for i in 0..hull.len() {
        let point = hull[i];
        let next = hull[(i + 1) % hull.len()];
        let prev = hull[(i + hull.len() - 1) % hull.len()];

        let edge_1 = next - point;
        let edge_2 = point - prev;
        let offset = (Vector2::new(edge_1.y, -edge_1.x).normalize()
            + Vector2::new(edge_2.y, -edge_2.x).normalize())
        .normalize();

        builder.add_vertex(point.push(0.0));
        builder.add_vertex((point + offset * raft_offset).push(raft_height));
    }

    let verts = builder.next_idx() - idx;
    for i in (0..verts).step_by(2) {
        if i != 0 && i + 3 < verts {
            builder.add_face([idx, idx + i + 2, idx + i]);
            builder.add_face([idx + 1, idx + i + 1, idx + i + 3]);
        }

        builder.add_quad([
            idx + i % verts,
            idx + (i + 1) % verts,
            idx + (i + 2) % verts,
            idx + (i + 3) % verts,
        ]);
    }
}
