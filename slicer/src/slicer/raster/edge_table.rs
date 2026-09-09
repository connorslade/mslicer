// Reference: https://www.cs.rit.edu/~icss571/filling/how_to.html

use std::collections::VecDeque;

use itertools::Itertools;
use nalgebra::Vector2;
use ordered_float::OrderedFloat;

use crate::slicer::raster::Segment;

#[derive(Debug)]
pub struct Edge {
    // Original edge geometry in pixel space.
    pub p_min: Vector2<f32>,
    pub p_max: Vector2<f32>,

    pub inv_slope: f32,
    pub entering: bool,
    pub priority: u8,
    pub exposure: u8,
}

#[derive(Debug)]
pub struct ActiveEdge {
    // Intersection with the center of the current pixel row.
    pub x: f32,
    pub y_max: f32,

    pub inv_slope: f32,
    pub entering: bool,
    pub priority: u8,
    pub exposure: u8,
}

pub fn global_edge_table(segments: impl Iterator<Item = Segment>) -> VecDeque<Edge> {
    let mut edges = Vec::new();
    for Segment {
        endpoints: [p0, p1],
        entering,
        priority,
        exposure,
    } in segments
    {
        let delta = p1 - p0;

        let (mut t_vals, mut t_len) = ([0.0, 1.0, 0.0, 0.0], 2);
        let mut add_t = |t: f32| {
            if (0.0..=1.0).contains(&t) {
                t_vals[t_len] = t;
                t_len += 1;
            }
        };

        (delta.x != 0.0).then(|| add_t(-p0.x / delta.x));
        (delta.y != 0.0).then(|| add_t(-p0.y / delta.y));

        let t_slice = &mut t_vals[..t_len];
        t_slice.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for (t0, t1) in t_slice.iter().tuple_windows() {
            let [p0, p1] = [t0, t1].map(|&t| (p0 + delta * t).map(|x| x.max(0.0)));
            if p0.y == p1.y {
                continue;
            }

            let inv_slope = (p1.x - p0.x) / (p1.y - p0.y);
            let (p_min, p_max) = if p0.y < p1.y { (p0, p1) } else { (p1, p0) };

            edges.push(Edge {
                p_min,
                p_max,
                inv_slope,
                entering,
                priority,
                exposure,
            });
        }
    }

    edges.sort_by(|a, b| {
        a.p_min
            .y
            .total_cmp(&b.p_min.y)
            .then_with(|| a.p_min.x.total_cmp(&b.p_min.x))
    });
    VecDeque::from(edges)
}

pub fn update_active_edges(edges: &mut VecDeque<Edge>, active: &mut Vec<ActiveEdge>, y: u32) {
    let scan_y = y as f32 + 0.5;

    // Use a half-open interval: p_min.y <= scan_y < p_max.y.
    active.retain(|e| e.y_max > scan_y);

    // Existing active edges advance by one complete scanline.
    active.iter_mut().for_each(|e| e.x += e.inv_slope);

    // Add edges intersecting the center of the current pixel row.
    while !edges.is_empty() && edges[0].p_min.y <= scan_y {
        let edge = edges.pop_front().unwrap();

        // This edge does not intersect any sampled scanline.
        if edge.p_max.y <= scan_y {
            continue;
        }

        let x = edge.p_min.x + (scan_y - edge.p_min.y) * edge.inv_slope;

        active.push(ActiveEdge {
            x,
            y_max: edge.p_max.y,
            inv_slope: edge.inv_slope,
            entering: edge.entering,
            priority: edge.priority,
            exposure: edge.exposure,
        });
    }

    active.sort_by_key(|e| OrderedFloat(e.x));
}
