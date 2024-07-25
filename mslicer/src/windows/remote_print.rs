use egui::{vec2, Align, Context, Grid, Layout, ProgressBar, Separator, TextEdit, Ui};
use remote_send::status::{FileTransferStatus, PrintInfoStatus};

use crate::app::App;

enum Action {
    None,
    Remove(usize),
}

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    if !app.remote_print.is_initialized() {
        ui.label("Remote print services have not been initialized.");
        ui.add_space(8.0);

        ui.vertical_centered(|ui| {
            if ui.button("Initialize").clicked() {
                app.remote_print.init().unwrap();
            }
        });

        return;
    }

    let mut action = Action::None;

    ui.heading("Printers");
    let printers = app.remote_print.printers();
    if printers.is_empty() {
        ui.label("No printers have been added yet.");
    }

    for (i, printer) in printers.iter().enumerate() {
        ui.with_layout(
            Layout::left_to_right(Align::Min).with_main_justify(true),
            |ui| {
                ui.horizontal(|ui| {
                    ui.strong(&printer.data.attributes.name);
                    ui.monospace(&printer.data.attributes.mainboard_id);
                });

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if ui.button("🗑 Delete").clicked() {
                        action = Action::Remove(i);
                    }
                    ui.add_space(ui.available_width());
                })
            },
        );

        Grid::new(format!("printer_{}", printer.data.attributes.mainboard_id))
            .num_columns(2)
            .striped(true)
            .with_row_color(|row, style| (row % 2 == 0).then_some(style.visuals.faint_bg_color))
            .show(ui, |ui| {
                ui.label("Firmware Version");
                ui.with_layout(
                    Layout::left_to_right(Align::Min)
                        .with_main_justify(true)
                        .with_main_align(Align::Min),
                    |ui| {
                        ui.monospace(&printer.data.attributes.firmware_version);
                    },
                );
                ui.end_row();

                ui.label("Resolution");
                let resolution = &printer.data.attributes.resolution;
                ui.monospace(format!("{}x{}", resolution.x, resolution.y));
                ui.end_row();

                ui.label("Capabilities");
                ui.monospace(
                    &printer
                        .data
                        .attributes
                        .capabilities
                        .iter()
                        .map(|x| format!("{x:?}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                ui.end_row();

                ui.label("Last Status");
                ui.monospace(&printer.last_update.format("%Y-%m-%d %H:%M:%S").to_string());
            });

        let print_info = &printer.data.status.print_info;
        if !matches!(
            print_info.status,
            PrintInfoStatus::None | PrintInfoStatus::Complete
        ) {
            ui.horizontal(|ui| {
                ui.label("Printing ");
                ui.monospace(&print_info.filename);
                ui.label(format!(". ({:?})", print_info.status));
            });
            ui.add(
                ProgressBar::new(print_info.current_layer as f32 / print_info.total_layer as f32)
                    .desired_width(ui.available_width()),
            );
        }

        let file_transfer = &printer.data.status.file_transfer_info;
        if file_transfer.status == FileTransferStatus::None && file_transfer.file_total_size != 0 {
            ui.horizontal(|ui| {
                ui.label("Transferring ");
                ui.monospace(&file_transfer.filename);
                ui.label(".");
            });
            ui.add(
                ProgressBar::new(
                    file_transfer.download_offset as f32 / file_transfer.file_total_size as f32,
                )
                .desired_width(ui.available_width()),
            );
        }

        if i + 1 != printers.len() {
            ui.separator();
        }
    }
    drop(printers);

    ui.add_space(16.0);
    ui.heading("Add Printer");
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        let scan = ui.button("Scan");
        let height = scan.rect.height();
        if scan.clicked() {
            app.dialog_builder()
                .with_title("Unimplemented")
                .with_body("Printer scanning is not implemented yet.")
                .open();
        }

        ui.add_sized(vec2(2.0, height), Separator::default());
        if ui.button("Connect").clicked() {
            app.remote_print
                .add_printer(&app.state.working_address)
                .unwrap();
            app.state.working_address.clear();
        }

        ui.add_sized(
            vec2(ui.available_width(), height),
            TextEdit::singleline(&mut app.state.working_address)
                .hint_text("192.168.1.233")
                .desired_width(ui.available_width()),
        );
    });

    match action {
        Action::Remove(i) => app.remote_print.remove_printer(i).unwrap(),
        Action::None => {}
    }
}
