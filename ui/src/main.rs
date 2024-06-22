use anyhow::Result;
use eframe::NativeOptions;
use egui::{IconData, Vec2, ViewportBuilder};
use wgpu::TextureFormat;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

mod app;
mod components;
mod windows;
mod workspace;
use app::App;

const ICON: &[u8] = include_bytes!("assets/icon.png");

fn main() -> Result<()> {
    let icon = image::load_from_memory(ICON)?;
    eframe::run_native(
        "mslicer",
        NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size(Vec2::new(1920.0, 1080.0))
                .with_icon(IconData {
                    rgba: icon.to_rgba8().to_vec(),
                    width: icon.width(),
                    height: icon.height(),
                }),
            depth_buffer: 24,
            stencil_buffer: 8,
            ..Default::default()
        },
        Box::new(|cc| {
            workspace::init_wgpu(cc);
            Box::new(App::default())
        }),
    )
    .unwrap();

    Ok(())
}
