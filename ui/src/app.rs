use std::sync::{Arc, RwLock};

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

    pub show_about: bool,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("mslicer");
                ui.separator();

                ui.menu_button("🖹 File", |ui| {
                    if ui.button("Import Modal").clicked() {
                        FileDialog::new()
                            .add_filter("STL", &["stl"])
                            .pick_file()
                            .map(|path| {
                                let mut file = std::fs::File::open(path).unwrap();
                                let modal = slicer::mesh::load_mesh(&mut file, "stl").unwrap();
                                self.meshes
                                    .write()
                                    .unwrap()
                                    .push(RenderedMesh::from_mesh(modal));
                            });
                    }

                    ui.separator();

                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory_mut(|mem| mem.reset_areas());
                    }
                });

                self.show_about ^= ui.button("About").clicked();
            });
        });

        Window::new("Transform").show(ctx, |ui| {
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

            ui.add(Slider::new(&mut self.camera.fov, 0.0..=2.0 * std::f32::consts::PI).text("FOV"));
            ui.add(Slider::new(&mut self.camera.near, 0.0..=10.0).text("Near"));
            ui.add(Slider::new(&mut self.camera.far, 0.0..=100.0).text("Far"));
        });

        Window::new("Slice Config").show(ctx, |ui| {
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

        if self.show_about {
            Window::new("About").show(ctx, |ui| {
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
        }

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

            meshes: Arc::new(RwLock::new(Vec::new())),
            show_about: false,
        }
    }
}
