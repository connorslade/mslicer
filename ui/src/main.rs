use anyhow::Result;
use eframe::NativeOptions;
use egui::Vec2;
use wgpu::TextureFormat;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

mod app;
mod components;
mod windows;
mod workspace;
use app::App;
use workspace::render;

fn main() -> Result<()> {
    eframe::run_native(
        "mslicer",
        NativeOptions {
            window_builder: Some(Box::new(|builder| {
                builder.with_inner_size(Vec2::new(1920.0, 1080.0))
            })),

            depth_buffer: 24,
            stencil_buffer: 8,
            ..Default::default()
        },
        Box::new(|cc| {
            render::init_wgpu(cc);
            Box::new(App::default())
        }),
    )
    .unwrap();

    Ok(())
}
