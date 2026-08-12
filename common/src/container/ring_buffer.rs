//! A Ring Buffer implementation.
//! A ring buffer is a type of buffer with a set length that will
//! overwrite the oldest values it holds when new ones are added.
//!
//! Modified from my [radio-data](https://github.com/connorslade/radio-data/blob/master/src/misc/ring_buffer.rs) project.

use core::mem::MaybeUninit;

/// Ring buffer that can hold any type.
/// The size of the buffer is defined as SIZE at compile time so it can be stored on the stack.
pub struct RingBuffer<T, const SIZE: usize> {
    pub data: [MaybeUninit<T>; SIZE],
    pub index: usize,
    pub filled: bool,
}

impl<T: Default + Copy, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Create a new RingBuffer using T::default().
    pub fn empty() -> Self {
        Self {
            data: [const { MaybeUninit::uninit() }; SIZE],
            index: 0,
            filled: false,
        }
    }
}

impl<T, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Adds a new value to the buffer
    pub fn push(&mut self, val: T) {
        self.data[self.index].write(val);
        let idx = self.index + 1;
        self.index = idx % SIZE;
        self.filled |= idx == SIZE;
    }

    /// Wraps Self::real to provide a safe and convenient API.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (self.data[self.index..].iter())
            .take(if self.filled { SIZE } else { 0 })
            .chain(self.data[..self.index].iter())
            .map(|x| unsafe { x.assume_init_ref() })
    }

    pub fn last(&self) -> Option<&T> {
        let last_idx = (self.index + SIZE - 1) % SIZE;
        let last = &self.data[last_idx];

        (self.filled || self.index > 0).then(|| unsafe { last.assume_init_ref() })
    }
}
