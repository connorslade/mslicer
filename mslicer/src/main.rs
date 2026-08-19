#![windows_subsystem = "windows"]

use std::{env, fs::File, panic, sync::Arc, thread};

use anyhow::Result;
use eframe::NativeOptions;
use egui::{FontDefinitions, Vec2, ViewportBuilder};
use egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{filter, fmt::layer, layer::SubscriberExt, util::SubscriberInitExt};
use wgpu::{DeviceDescriptor, Features, Limits, TextureFormat};

const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth24PlusStencil8;
const VERSION: &str = env!("CARGO_PKG_VERSION");

mod app;
mod project;
mod render;
mod system;
mod task;
mod ui;
mod util;
mod windows;
use app::{App, config::Config};

use crate::{
    system::{arguments::Args, icon},
    task::update_check_if_scheduled,
};

fn main() -> Result<()> {
    // Don't print panics on threads that are handled by the task system.
    let old_panic = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        (thread::current().name() != Some("task_thread")).then(|| old_panic(panic));
    }));

    let filter = filter::Targets::new()
        .with_default(LevelFilter::OFF)
        .with_target("mslicer", LevelFilter::TRACE)
        .with_target("remote_print", LevelFilter::TRACE)
        .with_target("slicer", LevelFilter::TRACE)
        .with_target("tools", LevelFilter::TRACE);
    let format = tracing_subscriber::fmt::layer();
    let collector = egui_tracing::EventCollector::new();

    let config_dir = dirs::config_dir().unwrap().join("mslicer");
    let log_file = File::create(config_dir.join("mslicer.log")).unwrap();
    let file_layer = layer().with_writer(log_file);

    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .with(collector.clone())
        .with(file_layer)
        .init();
    info!("Starting mslicer v{VERSION}");

    let args = Args::parse();
    #[allow(unused_mut)]
    let mut config = Config::load_or_default(&config_dir);
    let max_buffer_size = config.render.max_buffer_size;

    #[cfg(windows)]
    crate::system::windows::check_install(&mut config, &args)?;
    #[cfg(not(windows))]
    if args.install {
        tracing::warn!("--install has no effect on non-windows platforms, ignoring.")
    }

    eframe::run_native(
        "mslicer",
        NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size(Vec2::new(1920.0, 1080.0))
                .with_drag_and_drop(true)
                .with_icon(icon()),
            depth_buffer: 24,
            stencil_buffer: 8,
            multisampling: 4,
            centered: true,
            wgpu_options: WgpuConfiguration {
                wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                    device_descriptor: Arc::new(move |_adapter| DeviceDescriptor {
                        label: None,
                        required_features: Features::POLYGON_MODE_LINE | Features::PUSH_CONSTANTS,
                        required_limits: Limits {
                            max_buffer_size,
                            max_push_constant_size: 4,
                            ..Limits::default()
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            // Fixes crash on macOS (Observed on MacBook with touchbar running Sequoia)
            // See https://github.com/emilk/egui/discussions/7857
            persist_window: false,
            ..Default::default()
        },
        Box::new(|cc| {
            let mut fonts = FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);

            egui_extras::install_image_loaders(&cc.egui_ctx);

            let mut app = App::new(render::init_wgpu(cc), config_dir, config, collector);
            update_check_if_scheduled(&mut app);
            args.open.start(&mut app);

            Ok(Box::new(app))
        }),
    )
    .unwrap();

    Ok(())
}
