use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use egui::{
    emath::Numeric, CentralPanel, DragValue, Frame, Grid, Sense, Slider, TopBottomPanel, Ui, Window,
};
use egui_wgpu::Callback;
use nalgebra::{Vector2, Vector3};
use rfd::FileDialog;
use slicer::slicer::{ExposureConfig, SliceConfig};

use crate::{camera::Camera, render::RenderedMesh, workspace::WorkspaceRenderCallback};

pub struct App {
    pub camera: Camera,
    pub slice_config: SliceConfig,
    pub meshes: Arc<RwLock<Vec<RenderedMesh>>>,

    fps: FpsTracker,

    pub show_about: bool,
    pub show_slice_config: bool,
    pub show_transform: bool,
    pub show_modals: bool,
    pub show_stats: bool,
}

struct FpsTracker {
    last_frame: Instant,
    last_frame_time: f32,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.fps.update();

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("mslicer");
                ui.separator();

                ui.menu_button("🖹 File", |ui| {
                    if ui.button("Import Modal").clicked() {
                        // TODO: async
                        if let Some(path) =
                            FileDialog::new().add_filter("STL", &["stl"]).pick_file()
                        {
                            let name = path.file_name().unwrap().to_str().unwrap().to_string();

                            let mut file = std::fs::File::open(path).unwrap();
                            let modal = slicer::mesh::load_mesh(&mut file, "stl").unwrap();

                            self.meshes
                                .write()
                                .unwrap()
                                .push(RenderedMesh::from_mesh(modal).with_name(name));
                        }
                    }
                });

                ui.menu_button("🖹 View", |ui| {
                    fn show_entry(ui: &mut Ui, name: &str, show: &mut bool) {
                        *show ^= ui
                            .button(format!("{} {name}", if *show { "👁" } else { "🗙" }))
                            .clicked();
                    }

                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory_mut(|mem| mem.reset_areas());
                    }

                    ui.separator();

                    show_entry(ui, "About", &mut self.show_about);
                    show_entry(ui, "Modals", &mut self.show_modals);
                    show_entry(ui, "Slice Config", &mut self.show_slice_config);
                    show_entry(ui, "Stats", &mut self.show_stats);
                    show_entry(ui, "Transform", &mut self.show_transform);
                });

                ui.separator();

                if ui.button("Slice!").clicked() {
                    unimplemented!();
                }
            });
        });

        Window::new("Transform")
            .open(&mut self.show_transform)
            .default_width(0.0)
            .show(ctx, |ui| {
                ui.add(Slider::new(&mut self.camera.pos.x, -10.0..=10.0).text("X"));
                ui.add(Slider::new(&mut self.camera.pos.y, -10.0..=10.0).text("Y"));
                ui.add(Slider::new(&mut self.camera.pos.z, -10.0..=10.0).text("Z"));

                ui.separator();

                ui.add(
                    Slider::new(
                        &mut self.camera.pitch,
                        -std::f32::consts::PI..=std::f32::consts::PI,
                    )
                    .text("Pitch"),
                );
                ui.add(
                    Slider::new(
                        &mut self.camera.yaw,
                        -std::f32::consts::PI..=std::f32::consts::PI,
                    )
                    .text("Yaw"),
                );

                ui.separator();

                ui.add(
                    Slider::new(&mut self.camera.fov, 0.0..=2.0 * std::f32::consts::PI).text("FOV"),
                );
                ui.add(Slider::new(&mut self.camera.near, 0.0..=10.0).text("Near"));
                ui.add(Slider::new(&mut self.camera.far, 0.0..=100.0).text("Far"));
            });

        Window::new("Slice Config")
            .open(&mut self.show_slice_config)
            .show(ctx, |ui| {
                Grid::new("slice_config")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Platform Resolution");
                        vec2_dragger::<u32>(ui, self.slice_config.platform_resolution.as_mut());
                        ui.end_row();

                        ui.label("Platform Size");
                        vec3_dragger::<f32>(ui, self.slice_config.platform_size.as_mut());
                        ui.end_row();

                        ui.label("Slice Height");
                        ui.add(DragValue::new(&mut self.slice_config.slice_height));
                        ui.end_row();

                        ui.label("First Layers");
                        ui.add(DragValue::new(&mut self.slice_config.first_layers));
                        ui.end_row();
                    });

                ui.collapsing("Exposure Config", |ui| {
                    exposure_config_grid(ui, &mut self.slice_config.exposure_config);
                });

                ui.collapsing("First Exposure Config", |ui| {
                    exposure_config_grid(ui, &mut self.slice_config.first_exposure_config);
                });
            });

        Window::new("Modals")
            .open(&mut self.show_modals)
            .show(ctx, |ui| {
                let mut meshes = self.meshes.write().unwrap();

                if meshes.is_empty() {
                    ui.label("No modals loaded yet.");
                    return;
                }

                Grid::new("modals")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for (i, mesh) in meshes.iter_mut().enumerate() {
                            ui.label(&mesh.name);

                            ui.horizontal(|ui| {
                                mesh.hidden ^=
                                    ui.button(if mesh.hidden { "🗙" } else { "👁" }).clicked();

                                ui.collapsing("Details", |ui| {
                                    Grid::new(format!("modal_{}", i))
                                        .num_columns(2)
                                        .spacing([40.0, 4.0])
                                        .striped(true)
                                        .show(ui, |ui| {
                                            ui.label("Vertices");
                                            ui.monospace(mesh.mesh.vertices.len().to_string());
                                            ui.end_row();

                                            ui.label("Triangles");
                                            ui.monospace(mesh.mesh.faces.len().to_string());
                                            ui.end_row();
                                        });
                                });
                            });
                            ui.end_row()
                        }
                    });
            });

        Window::new("Stats")
            .open(&mut self.show_stats)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Frame Time: {:.2}ms",
                    self.fps.frame_time() * 1000.0
                ));
                ui.label(format!("FPS: {:.2}", 1.0 / self.fps.frame_time()));
            });

        Window::new("About")
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                ui.monospace(concat!("mslicer v", env!("CARGO_PKG_VERSION")));
                ui.label(
                    "A work in progress FOSS slicer for resin printers. Created by Connor Slade.",
                );
                ui.horizontal(|ui| {
                    ui.label("Github:");
                    ui.hyperlink_to(
                        "@connorslade/mslicer",
                        "https://github.com/connorslade/mslicer",
                    );
                });
            });

        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let (rect, _response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());

                let callback = Callback::new_paint_callback(
                    rect,
                    WorkspaceRenderCallback {
                        transform: self
                            .camera
                            .view_projection_matrix(rect.width() / rect.height()),
                        modals: self.meshes.clone(),
                    },
                );
                ui.painter().add(callback);
            });
    }
}

impl FpsTracker {
    fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            last_frame_time: 0.0,
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now - self.last_frame;
        self.last_frame_time = elapsed.as_secs_f32();
        self.last_frame = now;
    }

    fn frame_time(&self) -> f32 {
        self.last_frame_time
    }
}

fn dragger<Num: Numeric>(
    ui: &mut Ui,
    label: &str,
    value: &mut Num,
    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        ui.add(func(DragValue::new(value)));
        ui.label(label);
    });
}

fn vec2_dragger<Num: Numeric>(ui: &mut Ui, val: &mut [Num; 2]) {
    ui.horizontal(|ui| {
        ui.add(DragValue::new(&mut val[0]));
        ui.label("x");
        ui.add(DragValue::new(&mut val[1]));
    });
}

fn vec3_dragger<Num: Numeric>(ui: &mut Ui, val: &mut [Num; 3]) {
    ui.horizontal(|ui| {
        ui.add(DragValue::new(&mut val[0]));
        ui.label("x");
        ui.add(DragValue::new(&mut val[1]));
        ui.label("x");
        ui.add(DragValue::new(&mut val[2]));
    });
}

fn exposure_config_grid(ui: &mut Ui, config: &mut ExposureConfig) {
    Grid::new("exposure_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Exposure Time (s)");
            ui.add(DragValue::new(&mut config.exposure_time).clamp_range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Lift Distance (mm)");
            ui.add(DragValue::new(&mut config.lift_distance).clamp_range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Lift Speed (cm/min)");
            ui.add(DragValue::new(&mut config.lift_speed).clamp_range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Retract Distance (mm)");
            ui.add(DragValue::new(&mut config.retract_distance).clamp_range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Retract Speed (cm/min)");
            ui.add(DragValue::new(&mut config.retract_speed).clamp_range(0.0..=f32::MAX));
            ui.end_row();
        });
}

impl Default for App {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            slice_config: SliceConfig {
                platform_resolution: Vector2::new(11_520, 5_120),
                platform_size: Vector3::new(218.88, 122.904, 260.0),
                slice_height: 0.05,

                exposure_config: ExposureConfig {
                    exposure_time: 3.0,
                    ..Default::default()
                },
                first_exposure_config: ExposureConfig {
                    exposure_time: 50.0,
                    ..Default::default()
                },
                first_layers: 10,
            },
            fps: FpsTracker::new(),

            meshes: Arc::new(RwLock::new(Vec::new())),
            show_about: false,
            show_modals: false,
            show_slice_config: true,
            show_stats: false,
            show_transform: true,
        }
    }
}
