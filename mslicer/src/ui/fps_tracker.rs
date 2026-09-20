use std::time::Instant;

use common::container::RingBuffer;

pub struct FpsTracker {
    last_frame: Instant,
    history: RingBuffer<f32, 600>,
}

impl FpsTracker {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            history: RingBuffer::empty(),
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now - self.last_frame;
        self.history.push(elapsed.as_secs_f32());
        self.last_frame = now;
    }

    pub fn frame_time(&self) -> f32 {
        self.history.last().copied().unwrap_or_default()
    }

    pub fn fps_history(&self) -> impl Iterator<Item = f32> {
        self.history.iter().map(|x| x.recip())
    }
}
