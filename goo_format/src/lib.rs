mod default;
mod encoded_layer;
mod file;
mod header_info;
mod layer_content;
mod preview_image;

pub use encoded_layer::{LayerDecoder, LayerEncoder, Run};
pub use file::File;
pub use header_info::HeaderInfo;
pub use layer_content::LayerContent;
pub use preview_image::PreviewImage;

const ENDING_STRING: &[u8] = &[
    0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x44, 0x4C, 0x50, 0x00,
];
const MAGIC_TAG: &[u8] = &[0x07, 0x00, 0x00, 0x00, 0x44, 0x4C, 0x50, 0x00];
const DELIMITER: &[u8] = &[0xD, 0xA];
