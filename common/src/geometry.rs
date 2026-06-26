use nalgebra::Vector2;

// Andrew's monotone chain convex hull algorithm
// Reference: https://en.wikibooks.org/wiki/Algorithm_Implementation/Geometry/Convex_hull/Monotone_chain
pub fn convex_hull(points: &[Vector2<f32>]) -> Vec<Vector2<f32>> {
    let mut points = points.to_vec();
    points.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));

    let mut out = Vec::new();
    for point in points.iter().copied() {
        while out.len() >= 2 && cross(out[out.len() - 2], out[out.len() - 1], point) <= 0.0 {
            out.pop();
        }
        out.push(point);
    }

    out.pop();
    let lower = out.len();

    for i in (0..points.len()).rev() {
        while out.len() - lower >= 2
            && cross(out[out.len() - 2], out[out.len() - 1], points[i]) <= 0.0
        {
            out.pop();
        }
        out.push(points[i]);
    }

    out.pop();
    out
}

fn cross(o: Vector2<f32>, a: Vector2<f32>, b: Vector2<f32>) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}
