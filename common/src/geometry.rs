use nalgebra::Vector2;
use ordered_float::OrderedFloat;

pub fn convex_hull(points: &[Vector2<f32>]) -> Vec<&Vector2<f32>> {
    let first = points.iter().min_by_key(|p| OrderedFloat(p.x)).unwrap();

    let mut hull = vec![first];
    let mut current = first;

    loop {
        let mut next = current;
        for point in points {
            if *point == *current {
                continue;
            }

            if *next == *current || is_left_turn(current, next, point) {
                next = point;
            }
        }

        if *next == *first {
            break;
        }

        hull.push(next);
        current = next;
    }

    hull
}

fn is_left_turn(a: &Vector2<f32>, b: &Vector2<f32>, c: &Vector2<f32>) -> bool {
    let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    cross > 0.0 || (cross == 0.0 && (a - c).magnitude_squared() > (a - b).magnitude_squared())
}
