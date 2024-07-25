use egui::{vec2, Align, Context, Layout, Separator, TextEdit, Ui};

use crate::app::App;

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

    ui.heading("Printers");
    let printers = app.remote_print.printers();
    if printers.is_empty() {
        ui.label("No printers have been added yet.");
    }

    for (i, printer) in printers.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.strong(&printer.data.attributes.name);
            ui.monospace(&printer.data.attributes.mainboard_id);
        });

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
}
