use std::{sync::atomic::Ordering, time::Duration};

use chrono::DateTime;
use common::misc::human_duration;
use egui::{
    vec2, Align, Context, DragValue, Grid, Layout, ProgressBar, Separator, Spinner, TextEdit, Ui,
};
use notify_rust::Notification;
use remote_send::status::{FileTransferStatus, PrintInfoStatus};

use crate::app::App;

enum Action {
    None,
    Remove(usize),
}

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    if !app.remote_print.is_initialized() {
        ui.heading("Initialization");
        ui.label("Remote print services have not been initialized. Because multiple network servers are required, this feature is disabled by default for security reasons.");
        ui.add_space(8.0);

        ui.vertical_centered(|ui| {
            if ui.button("Initialize").clicked() | app.config.init_remote_print_at_startup {
                app.remote_print.init().unwrap();
                app.remote_print
                    .set_network_timeout(Duration::from_secs_f32(app.config.network_timeout));
            }
        });
    } else {
        let mqtt = app.remote_print.mqtt();
        let mut action = Action::None;

        ui.heading("Printers");
        let printers = app.remote_print.printers();
        if printers.is_empty() {
            ui.label("No printers have been added yet.");
        }

        for (i, printer) in printers.iter().enumerate() {
            let client = mqtt.get_client(&printer.mainboard_id);
            let attributes = &client.attributes;

            let last_update = client.last_update.load(Ordering::Relaxed);
            let last_update = DateTime::from_timestamp(last_update, 0).unwrap();

            ui.with_layout(
                Layout::left_to_right(Align::Min).with_main_justify(true),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(&attributes.name);
                        ui.monospace(&attributes.mainboard_id);
                    });

                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if ui.button("🗑 Delete").clicked() {
                            action = Action::Remove(i);
                        }
                        ui.add_space(ui.available_width());
                    })
                },
            );

            let status = client.status.lock();

            let print_info = &status.print_info;
            let printing = !matches!(
                print_info.status,
                PrintInfoStatus::None | PrintInfoStatus::Complete
            );
            if printing {
                app.state.send_print_completion = false;
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("Printing ");
                    ui.monospace(&print_info.filename);
                    ui.label(format!("({:?})", print_info.status));
                });

                if print_info.total_ticks != 0 {
                    let eta = human_duration(Duration::from_millis(
                        (print_info.total_ticks - print_info.current_ticks) as u64,
                    ));
                    ui.label(format!("ETA: {eta}"));
                }

                ui.add(
                    ProgressBar::new(
                        print_info.current_layer as f32 / print_info.total_layer as f32,
                    )
                    .text(format!(
                        "{}/{}",
                        print_info.current_layer, print_info.total_layer
                    ))
                    .desired_width(ui.available_width()),
                );
                ui.add_space(8.0);
            }

            if print_info.status == PrintInfoStatus::Complete
                && app.config.alert_print_completion
                && !app.state.send_print_completion
            {
                app.state.send_print_completion = true;
                Notification::new()
                    .summary("Print Complete")
                    .body(&format!(
                        "Printer `{}` has finished printing `{}`.",
                        attributes.name, print_info.filename
                    ))
                    .show()
                    .unwrap();
            }

            let file_transfer = &status.file_transfer_info;
            if file_transfer.status == FileTransferStatus::None
                && file_transfer.file_total_size != 0
            {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("Transferring ");
                    ui.monospace(&file_transfer.filename);
                });
                ui.add(
                    ProgressBar::new(
                        file_transfer.download_offset as f32 / file_transfer.file_total_size as f32,
                    )
                    .show_percentage()
                    .desired_width(ui.available_width()),
                );
                ui.add_space(8.0);
            }

            if file_transfer.status == FileTransferStatus::Done && !printing {
                ui.add_space(8.0);
                ui.label("File transfer complete.");
                if ui.button("Print").clicked() {
                    app.remote_print
                        .print(&attributes.mainboard_id, &file_transfer.filename)
                        .unwrap();
                }
                ui.add_space(8.0);
            }

            ui.add_space(8.0);
            Grid::new(format!("printer_{}", attributes.mainboard_id))
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
                            ui.monospace(&attributes.firmware_version);
                        },
                    );
                    ui.end_row();

                    ui.label("Resolution");
                    let resolution = &attributes.resolution;
                    ui.monospace(format!("{}x{}", resolution.x, resolution.y));
                    ui.end_row();

                    ui.label("Capabilities");
                    ui.monospace(
                        &attributes
                            .capabilities
                            .iter()
                            .map(|x| format!("{x:?}"))
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    ui.end_row();

                    ui.label("Last Status");
                    ui.monospace(&last_update.format("%Y-%m-%d %H:%M:%S").to_string());
                });

            if i + 1 != printers.len() {
                ui.separator();
            }
        }
        drop(printers);

        ui.add_space(16.0);
        ui.heading("Add Printer");
        ui.add_enabled_ui(!app.state.remote_print_connecting, |ui| {
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
                    app.state.remote_print_connecting = true;
                    app.remote_print
                        .add_printer(&app.state.working_address)
                        .unwrap();
                }

                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    if app.state.remote_print_connecting {
                        ui.add(Spinner::new());
                    }

                    ui.add_sized(
                        vec2(ui.available_width(), height),
                        TextEdit::singleline(&mut app.state.working_address)
                            .hint_text("192.168.1.233")
                            .desired_width(ui.available_width()),
                    );
                });
            });
        });

        match action {
            Action::Remove(i) => app.remote_print.remove_printer(i).unwrap(),
            Action::None => {}
        }
    }

    ui.add_space(16.0);
    ui.heading("Config");

    ui.checkbox(
        &mut app.config.alert_print_completion,
        "Send toast on print complete",
    );

    ui.checkbox(
        &mut app.config.init_remote_print_at_startup,
        "Initialize remote print at startup",
    );

    let last_timeout = app.config.network_timeout;
    ui.horizontal(|ui| {
        ui.add(
            DragValue::new(&mut app.config.network_timeout)
                .suffix("s")
                .max_decimals(1)
                .speed(0.1)
                .clamp_range(0.1..=60.0),
        );
        ui.label("Network timeout");
    });

    if app.remote_print.is_initialized() && last_timeout != app.config.network_timeout {
        app.remote_print
            .set_network_timeout(Duration::from_secs_f32(app.config.network_timeout));
    }
}