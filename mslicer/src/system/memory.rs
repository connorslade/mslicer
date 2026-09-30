use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::HashMap,
    mem,
    sync::atomic::{AtomicUsize, Ordering},
};

use common::{container::Run, slice::Layer};
use nalgebra::Vector2;

use crate::core::{App, slice::result::GenericSliceResult};

pub struct TrackedAllocator {
    usage: AtomicUsize,
}

#[global_allocator]
pub static ALLOCATOR: TrackedAllocator = TrackedAllocator::new();

impl TrackedAllocator {
    const fn new() -> Self {
        Self {
            usage: AtomicUsize::new(0),
        }
    }

    pub fn usage(&self) -> usize {
        self.usage.load(Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.usage.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.usage.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.usage.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.usage.fetch_sub(layout.size(), Ordering::Relaxed);
        self.usage.fetch_add(new_size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

pub struct MemoryBreakdown {
    pub total: usize,

    pub models: usize,
    pub history: usize,
    pub sliced: usize,
}

impl MemoryBreakdown {
    pub fn create(app: &mut App) -> Self {
        let mut mesh = HashMap::new();
        for model in app.project.models.iter() {
            mesh.entry(model.mesh.mesh_id())
                .or_insert_with(|| model.mesh_memory_size());
        }

        let sliced = if let Some(slice_operation) = &app.slice_operation
            && let Some(result) = slice_operation.result().as_mut()
        {
            match &result.inner {
                GenericSliceResult::Raster(raster) => (raster.layers.iter())
                    .map(|x| x.data.capacity() * mem::size_of::<Run>() + mem::size_of::<Layer>())
                    .sum(),
                GenericSliceResult::Vector(vector) => (vector.layers.iter())
                    .flat_map(|x| x.iter())
                    .map(|x| x.capacity() * mem::size_of::<Vector2<f32>>())
                    .sum(),
            }
        } else {
            0
        };

        Self {
            total: ALLOCATOR.usage(),
            history: app.history.memory_size(&mesh),
            models: mesh.values().sum(),
            sliced,
        }
    }

    pub fn remaining(&self) -> usize {
        let tracked = self.models + self.history + self.sliced;
        self.total.saturating_sub(tracked)
    }
}
