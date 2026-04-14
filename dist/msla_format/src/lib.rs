#![doc = include_str!("../README.md")]

#[cfg(feature = "ctb")]
pub mod ctb;
#[cfg(feature = "goo")]
pub mod goo;
#[cfg(feature = "nanodlp")]
pub mod nanodlp;

mod common;

use common::progress;
pub use common::{
    container,
    progress::Progress,
    serde,
    slice::{DynSlicedFile, EncodableLayer, SliceInfo, SlicedFile},
    units,
};
pub mod slice {
    //! Simplified configuration for slicing a model.
    pub(crate) use crate::common::slice::*;
    pub use crate::common::slice::{ExposureConfig, SliceConfig};
}
