use std::{fs::File, io::Write};

use egui::{Align, Button, ComboBox, DragValue, Layout, Ui, Widget, vec2};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular::{BOUNDING_BOX, EYE, INFO, TRASH};
use tools::printed_circuit_board::Alignment;

use crate::{
    app::App,
    generator_tool,
    task::{FileDialog, MultiFileDialog},
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str = "Use your MSLA resin printer to expose UV sensitive photoresist or soldermask for PCB manufacturing.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Printed Circuit Board", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.printed_circuit_board;

    ui.horizontal(|ui| {
        if ui.button("Load Gerbers").clicked() {
            app.tasks.add(MultiFileDialog::pick_files(
                ("Gerber", &["gbr"]),
                |app, paths, _tasks| {
                    (paths.iter()).for_each(|path| app.state.tools.printed_circuit_board.load(path))
                },
            ));
        }

        ui.add_enabled_ui(!tool.layers.is_empty(), |ui| {
            if ui.button("Export SVG").clicked() {
                let svg = tool.svg();
                app.tasks.add(FileDialog::save_file(
                    ("SVG", &["svg"]),
                    move |_app, path, _tasks| {
                        File::create(path.with_extension("svg"))
                            .unwrap()
                            .write_all(svg.as_bytes())
                            .unwrap();
                    },
                ));
            }
        });
    });

    if !tool.layers.is_empty() {
        ui.add_space(8.0);

        ui.separator();
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto().at_least(150.0))
            .column(Column::remainder())
            .header(16.0, |mut row| {
                row.col(|_| {});
                row.col(|ui| {
                    ui.label(EYE).on_hover_text("Layer visible");
                });
                row.col(|ui| {
                    ui.label(BOUNDING_BOX).on_hover_text("Part of bounding box");
                });

                for label in ["Layer", "Project"] {
                    row.col(|ui| {
                        ui.label(label);
                    });
                }
            })
            .body(|mut body| {
                let mut delete = None;
                for (i, layer) in tool.layers.iter_mut().enumerate() {
                    body.row(16.0, |mut row| {
                        row.col(|ui| {
                            ui.style_mut().visuals.button_frame = false;
                            ui.button(TRASH).clicked().then(|| delete = Some(i));
                        });

                        row.col(|ui| {
                            ui.checkbox(&mut layer.mode.polygon, "");
                        });
                        row.col(|ui| {
                            ui.checkbox(&mut layer.mode.bounds, "");
                        });

                        row.col(|ui| {
                            ui.label(or_unknown(&layer.gerber.layer));
                        });

                        row.col(|ui| {
                            ui.label(or_unknown(&layer.gerber.name));
                        });
                    });
                }

                if let Some(idx) = delete {
                    tool.layers.remove(idx);
                }
            });

        ui.separator();
    }

    ui.add_space(8.0);
    grid("").show(ui, |ui| {
        ui.label("Exposure");
        ui.horizontal(|ui| {
            DragValue::new(tool.exposure_time.raw_mut())
                .suffix(" s")
                .speed(0.1)
                .range(0.0..=f32::MAX)
                .ui(ui);
            ui.take_available_width();
        });
        ui.end_row();

        ui.label("Alignment");
        alignment(ui, &mut tool.alignment);
        ui.end_row();

        ui.label("Photoresist");
        ComboBox::from_id_salt("photoresist")
            .selected_text(if tool.invert { "Positive" } else { "Negative" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut tool.invert, false, "Negative");
                ui.selectable_value(&mut tool.invert, true, "Positive");
            });
        ui.end_row();
    });

    ui.add_space(8.0);
    ui.collapsing("Offset", |ui| {
        ui.label("All offset distances are in millimeters.");

        grid("").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Pre Offset");
                ui.label(INFO).on_hover_text("Applied before flip");
            });
            ui.horizontal(|ui| {
                ui.add(DragValue::new(tool.pre_offset.x.raw_mut()).speed(0.1));
                ui.label("×");
                ui.add(DragValue::new(tool.pre_offset.y.raw_mut()).speed(0.1));
            });
            ui.end_row();

            ui.horizontal(|ui| {
                ui.label("Post Offset");
                ui.label(INFO).on_hover_text("Applied after flip");
            });
            ui.horizontal(|ui| {
                ui.add(DragValue::new(tool.post_offset.x.raw_mut()).speed(0.1));
                ui.label("×");
                ui.add(DragValue::new(tool.post_offset.y.raw_mut()).speed(0.1));
                ui.take_available_width();
            });
            ui.end_row();
        });
    });

    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        ui.checkbox(&mut tool.flip.enabled, "");
        ui.collapsing("Flip", |ui| {
            grid("").show(ui, |ui| {
                ui.label("Angle");
                DragValue::new(&mut tool.flip.angle).suffix("°").ui(ui);
                ui.end_row();

                ui.label("Offset");
                DragValue::new(tool.flip.offset.raw_mut())
                    .suffix(" mm")
                    .ui(ui);
                ui.end_row();

                ui.label("Alignment");
                ui.horizontal(|ui| {
                    alignment(ui, &mut tool.flip.alignment);
                    ui.take_available_width();
                });
                ui.end_row();
            });
        });
    });

    ui.add_space(8.0);
    ui.vertical_centered(|ui| {
        let button = Button::new("Generate").min_size(vec2(ui.available_width(), 0.0));
        if ui
            .add_enabled(!slicing && !tool.layers.is_empty(), button)
            .clicked()
        {
            generator_tool!(app, tool);
        }
    });

    false
}

fn alignment(ui: &mut Ui, value: &mut Alignment) {
    ComboBox::from_id_salt("alignment")
        .selected_text(value.name())
        .show_ui(ui, |ui| {
            for alignment in Alignment::ALL {
                ui.selectable_value(value, alignment, alignment.name());
            }
        });
}

fn or_unknown(val: &Option<String>) -> &str {
    val.as_ref().map(|x| x.as_str()).unwrap_or("Unknown")
}
