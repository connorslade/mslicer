use wgpu::TextureFormat;

const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth24PlusStencil8;

pub mod app;
pub mod plugins;
pub mod post_processing;
pub mod render;
pub mod ui;
pub mod windows;
