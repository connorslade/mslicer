//! Elegoo format V3.0 (`.goo`).
//!
//! ## References
//!
//! - [Official Format Spec](https://github.com/elegooofficial/GOO)

mod encoding;
mod file;
mod header;
mod layer;
mod preview;

pub use encoding::{LayerDecoder, LayerEncoder};
pub use file::File;
pub use header::{ExposureDelayMode, Header};
pub use layer::Layer;
pub use preview::PreviewImage;

const ENDING_STRING: &[u8] = &[
    0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x44, 0x4C, 0x50, 0x00,
];
const MAGIC_TAG: &[u8] = &[0x07, 0x00, 0x00, 0x00, 0x44, 0x4C, 0x50, 0x00];
const DELIMITER: &[u8] = &[0xD, 0xA];
