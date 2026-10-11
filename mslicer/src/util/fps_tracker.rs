use std::time::Instant;

use common::{
    container::RingBuffer,
    units::{Second, Seconds},
};

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

    pub fn avg_frame_time(&self, duration: Option<Seconds>) -> f32 {
        let max_time = match duration {
            Some(t) => t.get::<Second>(),
            None => f32::MAX,
        };

        let mut sum = 0.0;
        let mut n = 0;

        for frame_time in self.history.iter().rev() {
            sum += frame_time;
            n += 1;

            if sum >= max_time {
                break;
            }
        }

        if n == 0 { 0.0 } else { sum / n as f32 }
    }

    pub fn fps_history(&self) -> impl Iterator<Item = f32> {
        self.history.iter().map(|x| x.recip())
    }
}
