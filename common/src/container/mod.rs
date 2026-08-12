//! Container types.

mod bitvec;
mod clusters;
mod image;
mod ring_buffer;
pub mod rle;
pub use self::{
    bitvec::BitVec,
    clusters::{ArrayCluster, Clusters},
    image::{Image, ImageRuns},
    ring_buffer::RingBuffer,
    rle::Run,
};
