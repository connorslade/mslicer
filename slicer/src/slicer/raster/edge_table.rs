// Reference: https://www.cs.rit.edu/~icss571/filling/how_to.html

use std::collections::VecDeque;

use itertools::Itertools;
use nalgebra::Vector2;
use ordered_float::OrderedFloat;

use crate::slicer::raster::Segment;

#[derive(Debug)]
pub struct Edge {
    pub min: Vector2<u32>,

    pub y_max: u32,
    pub inv_slope: f32,
    pub entering: bool,
    pub priority: u8,
    pub exposure: u8,
}

#[derive(Debug)]
pub struct ActiveEdge {
    pub x: f32,

    pub y_max: u32,
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
            let pos = [p0, p1].map(|p| p.map(|x| x.round() as u32));
            if pos[0].y == pos[1].y {
                continue;
            }

            edges.push(Edge {
                min: pos[(pos[0].y >= pos[1].y) as usize],
                y_max: pos[0].y.max(pos[1].y),
                inv_slope,
                entering,
                priority,
                exposure,
            });
        }
    }

    edges.sort_by(|a, b| a.min.y.cmp(&b.min.y).then_with(|| a.min.x.cmp(&b.min.x)));
    VecDeque::from(edges)
}

pub fn update_active_edges(edges: &mut VecDeque<Edge>, active: &mut Vec<ActiveEdge>, y: u32) {
    active.retain(|x| x.y_max > y);
    active.iter_mut().for_each(|e| e.x += e.inv_slope);
    while !edges.is_empty() && edges[0].min.y == y {
        let edge = edges.pop_front().unwrap();
        active.push(ActiveEdge {
            x: edge.min.x as f32,
            y_max: edge.y_max,
            inv_slope: edge.inv_slope,
            entering: edge.entering,
            priority: edge.priority,
            exposure: edge.exposure,
        });
    }
    active.sort_by_key(|x| OrderedFloat(x.x));
}
