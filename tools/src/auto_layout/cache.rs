use std::{collections::HashMap, f32::consts::TAU, sync::Arc};

use common::geometry::convex_hull;
use itertools::Itertools;
use nalgebra::{Rotation2, Vector2};

use crate::auto_layout::bounds::Bounds2D;

const OFFSET_PRECISION: usize = 6;

// todo: cache line friendly ordering!
pub struct LayoutCache {
    hulls: HashMap<CacheEntry, Arc<Hull>>,
    nfps: HashMap<(CacheEntry, CacheEntry), Arc<Vec<Vector2<f32>>>>,

    padding: f32,
}

pub struct Hull {
    pub hull: Vec<Vector2<f32>>,
    pub bounds: Bounds2D,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct CacheEntry {
    mesh: usize, // Mesh id, not model id!
    rotation: u32,
}

impl Hull {
    pub fn new(hull: Vec<Vector2<f32>>) -> Self {
        Self {
            bounds: Bounds2D::new_containing(&hull),
            hull,
        }
    }
}

impl LayoutCache {
    pub fn new(padding: f32) -> Self {
        Self {
            hulls: HashMap::new(),
            nfps: HashMap::new(),
            padding,
        }
    }

    pub fn populate_hull(&mut self, entry: CacheEntry, hull: impl FnOnce() -> Hull) {
        if !self.hulls.contains_key(&entry) {
            self.hulls.insert(entry, Arc::new(hull()));
        }
    }

    pub fn hull(&mut self, entry: &CacheEntry) -> Arc<Hull> {
        if let Some(hull) = self.hulls.get(entry) {
            return hull.clone();
        }

        if let Some(hull) = self.hulls.get(&entry.with_rotation(0.0)) {
            let rotation = Rotation2::new(entry.rotation());

            let hull = hull.hull.iter().map(|x| rotation * *x).collect::<Vec<_>>();
            let bounds = Bounds2D::new_containing(&hull);

            let new = Arc::new(Hull { hull, bounds });
            self.hulls.insert(*entry, new.clone());
            return new;
        }

        panic!("Hull not found!")
    }

    pub fn nfp(&mut self, a: CacheEntry, b: CacheEntry) -> Arc<Vec<Vector2<f32>>> {
        if let Some(nfp) = self.nfps.get(&(a, b)) {
            return nfp.clone();
        }

        let (hull_a, hull_b) = (self.hull(&a), self.hull(&b));
        let nfp = non_fitting_polygon(hull_a.hull.iter(), hull_b.hull.iter());
        let nfp = Arc::new(offset(&nfp, self.padding, OFFSET_PRECISION));

        self.nfps.insert((a, b), nfp.clone());
        nfp
    }
}

impl CacheEntry {
    pub fn new(mesh: usize, rotation: f32) -> Self {
        Self {
            mesh,
            rotation: rotation.to_bits(),
        }
    }

    pub fn with_rotation(self, rotation: f32) -> Self {
        Self {
            rotation: rotation.to_bits(),
            ..self
        }
    }

    pub fn rotation(&self) -> f32 {
        f32::from_bits(self.rotation)
    }
}

fn non_fitting_polygon<'a>(
    a: impl Iterator<Item = &'a Vector2<f32>>,
    b: impl Iterator<Item = &'a Vector2<f32>> + Clone,
) -> Vec<Vector2<f32>> {
    let points = a
        .cartesian_product(b)
        .map(|(i, j)| *i - *j)
        .collect::<Vec<_>>();
    convex_hull(&points)
}

fn offset(points: &[Vector2<f32>], d: f32, n: usize) -> Vec<Vector2<f32>> {
    let points = (points.iter())
        .cartesian_product(disk(n, d).iter())
        .map(|(i, j)| *i + *j)
        .collect::<Vec<_>>();
    convex_hull(&points)
}

fn disk(n: usize, r: f32) -> Vec<Vector2<f32>> {
    (0..n)
        .map(|i| {
            let (y, x) = (i as f32 / n as f32 * TAU).sin_cos();
            Vector2::new(x, y) * r
        })
        .collect()
}
