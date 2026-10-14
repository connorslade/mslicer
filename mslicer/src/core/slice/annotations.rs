use std::{
    collections::HashMap,
    mem,
    sync::atomic::{AtomicBool, Ordering},
};

use common::container::Run;
use egui::Color32;
use parking_lot::{Mutex, MutexGuard};

pub const ISLAND_COLOR: Color32 = Color32::from_rgb(159, 44, 54);

#[derive(Default)]
pub struct Annotations {
    layers: Mutex<HashMap<usize, Vec<Run<Annotation>>>>,
    updated: AtomicBool,
}

pub struct LockedAnnotations<'a> {
    layers: MutexGuard<'a, HashMap<usize, Vec<Run<Annotation>>>>,
    updated: &'a AtomicBool,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Annotation {
    None = 0b00,
    Island = 0b01,
}

impl Annotations {
    pub fn lock(&self) -> LockedAnnotations<'_> {
        LockedAnnotations {
            layers: self.layers.lock(),
            updated: &self.updated,
        }
    }

    pub fn take_updated(&self) -> bool {
        self.updated.swap(false, Ordering::Relaxed)
    }
}

impl<'a> LockedAnnotations<'a> {
    pub fn contains(&self, layer: usize) -> bool {
        if let Some(layer) = self.layers.get(&layer) {
            layer.iter().any(|x| !matches!(x.value, Annotation::None))
        } else {
            false
        }
    }

    pub fn get_layer(&self, layer: usize) -> Vec<Run> {
        let Some(layer) = self.layers.get(&layer) else {
            return Vec::new();
        };

        // SAFETY: Annotation has repr(u8), so can be safely interpreted as a u8
        unsafe { mem::transmute::<Vec<Run<Annotation>>, Vec<Run<u8>>>(layer.clone()) }
    }

    pub fn insert_layer(&mut self, annotation: Annotation, layer: usize, runs: &[u64]) {
        assert!(!self.layers.contains_key(&layer)); // not yet implemented!

        let runs = runs
            .iter()
            .enumerate()
            .map(|(i, &l)| Run {
                length: l,
                value: [Annotation::None, annotation][(i % 2 != 0) as usize],
            })
            .collect::<Vec<_>>();
        self.layers.insert(layer, runs);
        self.updated.store(true, Ordering::Relaxed);
    }
}
