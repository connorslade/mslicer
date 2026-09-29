use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::core::App;

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

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.usage.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) };
    }
}

pub struct MemoryBreakdown {
    pub total: usize,
    pub models: usize,
}

impl MemoryBreakdown {
    pub fn create(app: &mut App) -> Self {
        let mut mesh = HashMap::new();
        for model in app.project.models.iter() {
            mesh.entry(model.mesh.mesh_id()).or_insert_with(|| {
                model.mesh.memory_size()
                    + (model.bvh.as_ref())
                        .map(|x| x.memory_size())
                        .unwrap_or_default()
                    + (model.half_edge.as_ref())
                        .map(|x| x.memory_size())
                        .unwrap_or_default()
            });
        }

        Self {
            total: ALLOCATOR.usage(),
            models: mesh.values().sum(),
        }
    }

    pub fn misc(&self) -> usize {
        self.total - self.models
    }
}
