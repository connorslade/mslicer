//! Container types.

mod bitvec;
mod clusters;
mod image;
pub mod rle;
pub use self::{
    bitvec::BitVec,
    clusters::{ArrayCluster, Clusters},
    image::{Image, ImageRuns},
    rle::Run,
};
