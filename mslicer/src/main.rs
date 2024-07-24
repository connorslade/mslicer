use std::sync::Arc;

use anyhow::Result;
use eframe::NativeOptions;
use egui::{IconData, Vec2, ViewportBuilder};
use egui_wgpu::WgpuConfiguration;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};
use wgpu::{DeviceDescriptor, Features, TextureFormat};

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

mod app;
mod components;
mod render;
mod slice_operation;
mod windows;
use app::App;

const ICON: &[u8] = include_bytes!("assets/icon.png");

fn main() -> Result<()> {
    let filter = filter::Targets::new()
        .with_default(LevelFilter::OFF)
        .with_target("mslicer", LevelFilter::TRACE);
    let format = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .init();

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
            wgpu_options: WgpuConfiguration {
                device_descriptor: Arc::new(|_adapter| DeviceDescriptor {
                    required_features: Features::POLYGON_MODE_LINE,
                    ..Default::default()
                }),
                ..Default::default()
            },
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
