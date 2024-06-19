use std::cmp::{max, min};

use image::{ImageBuffer, Pixel, Rgb};
use imageproc::{
    drawing::{BresenhamLineIter, Canvas},
    point::Point,
};

pub fn draw_polygon_with_mut<L>(
    canvas: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    poly: &[Point<i32>],
    color: Rgb<u8>,
    plotter: L,
) where
    L: Fn(&mut ImageBuffer<Rgb<u8>, Vec<u8>>, (f32, f32), (f32, f32), Rgb<u8>),
{
    if poly.is_empty() {
        return;
    }
    if poly[0] == poly[poly.len() - 1] {
        panic!(
            "First point {:?} == last point {:?}",
            poly[0],
            poly[poly.len() - 1]
        );
    }

    let mut y_min = i32::MAX;
    let mut y_max = i32::MIN;
    for p in poly {
        y_min = min(y_min, p.y);
        y_max = max(y_max, p.y);
    }

    let (width, height) = canvas.dimensions();

    // Intersect polygon vertical range with image bounds
    y_min = max(0, min(y_min, height as i32 - 1));
    y_max = max(0, min(y_max, height as i32 - 1));

    let mut closed: Vec<Point<i32>> = poly.to_vec();
    closed.push(poly[0]);

    let edges: Vec<&[Point<i32>]> = closed.windows(2).collect();
    let mut intersections = Vec::new();

    for y in y_min..y_max + 1 {
        for edge in &edges {
            let p0 = edge[0];
            let p1 = edge[1];

            if p0.y <= y && p1.y >= y || p1.y <= y && p0.y >= y {
                if p0.y == p1.y {
                    // Need to handle horizontal lines specially
                    intersections.push(p0.x);
                    intersections.push(p1.x);
                } else if p0.y == y || p1.y == y {
                    if p1.y > y {
                        intersections.push(p0.x);
                    }
                    if p0.y > y {
                        intersections.push(p1.x);
                    }
                } else {
                    let fraction = (y - p0.y) as f32 / (p1.y - p0.y) as f32;
                    let inter = p0.x as f32 + fraction * (p1.x - p0.x) as f32;
                    intersections.push(inter.round() as i32);
                }
            }
        }

        intersections.sort_unstable();
        intersections.chunks(2).for_each(|range| {
            let mut from = min(range[0], width as i32);
            let mut to = min(range[1], width as i32 - 1);
            if from < width as i32 && to >= 0 {
                // draw only if range appears on the canvas
                from = max(0, from);
                to = max(0, to);

                for x in from..to + 1 {
                    let current = canvas.get_pixel(x as u32, y as u32);
                    if *current == color {
                        canvas.draw_pixel(x as u32, y as u32, Rgb([0, 0, 0]));
                    } else {
                        canvas.draw_pixel(x as u32, y as u32, color);
                    }
                }
            }
        });

        intersections.clear();
    }

    for edge in &edges {
        let start = (edge[0].x as f32, edge[0].y as f32);
        let end = (edge[1].x as f32, edge[1].y as f32);
        plotter(canvas, start, end, color);
    }
}

pub fn draw_line_segment_invert_mut(
    canvas: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    start: (f32, f32),
    end: (f32, f32),
    color: Rgb<u8>,
) {
    let (width, height) = canvas.dimensions();
    let in_bounds = |x, y| x >= 0 && x < width as i32 && y >= 0 && y < height as i32;

    let line_iterator = BresenhamLineIter::new(start, end);

    for point in line_iterator {
        let x = point.0;
        let y = point.1;

        if in_bounds(x, y) {
            let current = canvas.get_pixel(x as u32, y as u32);
            if *current == color {
                canvas.draw_pixel(x as u32, y as u32, Rgb([0, 0, 0]));
            } else {
                canvas.draw_pixel(x as u32, y as u32, color);
            }
        }
    }
}
