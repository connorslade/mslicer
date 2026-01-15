#![windows_subsystem = "windows"]

use std::sync::Arc;

use anyhow::Result;
use eframe::NativeOptions;
use egui::{FontDefinitions, IconData, Vec2, ViewportBuilder};
use egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};
use wgpu::{DeviceDescriptor, Features, Limits, TextureFormat};

const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth24PlusStencil8;

mod app;
mod plugins;
mod render;
mod ui;
mod util;
mod windows;
use app::{App, config::Config};

const ICON: &[u8] = include_asset!("icon.png");

fn main() -> Result<()> {
    let filter = filter::Targets::new()
        .with_default(LevelFilter::OFF)
        .with_target("mslicer", LevelFilter::TRACE)
        .with_target("slicer", LevelFilter::TRACE)
        .with_target("remote_send", LevelFilter::TRACE);
    let format = tracing_subscriber::fmt::layer();
    let collector = egui_tracing::EventCollector::new();

    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .with(collector.clone())
        .init();

    let config_dir = dirs::config_dir().unwrap().join("mslicer");
    let config = Config::load_or_default(&config_dir);

    let max_buffer_size = config.max_buffer_size;
    let icon = image::load_from_memory(ICON)?;
    eframe::run_native(
        "mslicer",
        NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size(Vec2::new(1920.0, 1080.0))
                .with_drag_and_drop(true)
                .with_icon(IconData {
                    rgba: icon.to_rgba8().to_vec(),
                    width: icon.width(),
                    height: icon.height(),
                }),
            depth_buffer: 24,
            stencil_buffer: 8,
            multisampling: 4,
            wgpu_options: WgpuConfiguration {
                wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                    device_descriptor: Arc::new(move |_adapter| DeviceDescriptor {
                        label: None,
                        required_features: Features::POLYGON_MODE_LINE,
                        required_limits: Limits {
                            max_buffer_size,
                            ..Limits::default()
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| {
            let render_state = render::init_wgpu(cc);

            let mut fonts = FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);

            let app = App::new(render_state, config_dir, config, collector);
            Ok(Box::new(app))
        }),
    )
    .unwrap();

    Ok(())
}
