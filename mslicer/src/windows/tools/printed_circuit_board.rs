use std::{fs::File, io::Write};

use egui::{Button, ComboBox, DragValue, Ui, Widget};
use egui_phosphor::regular::{ANGLE, ARROWS_OUT_LINE_VERTICAL};
use tools::printed_circuit_board::Alignment;

use crate::{
    app::App,
    generator_tool,
    task::FileDialog,
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str = "Use your msla resin printer to expose photoresist for PCB etching.";

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
        if ui.button("Load Gerber").clicked() {
            app.tasks.add(FileDialog::pick_file(
                ("Gerber", &["gbr"]),
                |app, path, _tasks| app.state.tools.printed_circuit_board.load(path),
            ));
        }

        ui.add_enabled_ui(tool.gerber.is_some(), |ui| {
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

    if let Some(gerber) = &tool.gerber {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if let Some(name) = &gerber.name {
                ui.label(format!("Loaded {name}."));
            } else {
                ui.label("Loaded.");
            }

            if let Some(layer) = &gerber.layer {
                ui.label(format!("({layer})"));
            }
        });
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
        ComboBox::new("alignment", "")
            .selected_text(tool.alignment.name())
            .show_ui(ui, |ui| {
                for alignment in Alignment::ALL {
                    ui.selectable_value(&mut tool.alignment, alignment, alignment.name());
                }
            });
        ui.end_row();

        ui.horizontal(|ui| {
            ui.label("Flip");
            ui.checkbox(&mut tool.flip.enabled, "");
        });
        ui.horizontal(|ui| {
            ui.add_enabled_ui(tool.flip.enabled, |ui| {
                ui.label(ANGLE);
                DragValue::new(&mut tool.flip.angle).suffix("°").ui(ui);
                ui.separator();
                ui.label(ARROWS_OUT_LINE_VERTICAL);
                DragValue::new(tool.flip.offset.raw_mut())
                    .suffix(" mm")
                    .ui(ui);
            });
        });
        ui.end_row();

        ui.label("Offset (mm)");
        ui.horizontal(|ui| {
            ui.add(DragValue::new(tool.offset.x.raw_mut()).speed(0.1));
            ui.label("×");
            ui.add(DragValue::new(tool.offset.y.raw_mut()).speed(0.1));
        });
        ui.end_row();

        ui.label("Photoresist");
        ComboBox::new("photoresist", "")
            .selected_text(if tool.invert { "Positive" } else { "Negative" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut tool.invert, false, "Negative");
                ui.selectable_value(&mut tool.invert, true, "Positive");
            });
        ui.end_row();
    });
    ui.add_space(8.0);

    ui.centered_and_justified(|ui| {
        if ui
            .add_enabled(!slicing && tool.gerber.is_some(), Button::new("Generate"))
            .clicked()
        {
            generator_tool!(app, tool);
        }
    });

    false
}
