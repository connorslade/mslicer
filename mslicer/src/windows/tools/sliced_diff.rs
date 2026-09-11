use std::{fs::File, mem, sync::Arc};

use common::slice::format::RasterFormat;
use const_format::concatcp;
use egui::{Button, ComboBox, DragValue, RichText, Ui};
use egui_phosphor::regular::{FOLDER_OPEN, STACK_SIMPLE, SWAP};
use slicer::util;
use tools::sliced_diff::{Difference, Source, SourceId};

use crate::{
    app::{App, slice_operation::GenericSliceData},
    generator_tool,
    task::FileDialog,
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str =
    "Compares the layer data between two sliced files using your selected difference method.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Sliced Diff", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();

    if ui.button(concatcp!(SWAP, " Swap Files")).clicked() {
        let tool = &mut app.state.tools.sliced_diff;
        mem::swap(&mut tool.new, &mut tool.old);
    }

    grid("").show(ui, |ui| {
        ui.label("Old");
        source_ui(app, ui, SourceId::Old);
        ui.end_row();

        ui.label("New");
        source_ui(app, ui, SourceId::New);
        ui.end_row();

        let tool = &mut app.state.tools.sliced_diff;
        ui.label("Difference");
        ComboBox::from_id_salt("difference")
            .selected_text(tool.difference.name())
            .show_ui(ui, |ui| {
                for difference in Difference::ALL {
                    ui.selectable_value(&mut tool.difference, difference, difference.name());
                }
            });
        ui.end_row();

        ui.label("Threshold");
        ui.horizontal(|ui| {
            ui.checkbox(&mut tool.threshold.0, "");
            ui.add_enabled(tool.threshold.0, DragValue::new(&mut tool.threshold.1));
            ui.take_available_width();
        });
        ui.end_row();
    });

    let tool = &mut app.state.tools.sliced_diff;
    let has_source = tool.sources_specified();
    let sources_match = tool.matching_sources();

    ui.add_space(8.0);
    if has_source && !sources_match {
        ui.label("Layer resolutions don't match.");
        ui.add_space(8.0);
    }

    ui.centered_and_justified(|ui| {
        if ui
            .add_enabled(
                !slicing && has_source && sources_match,
                Button::new("Generate"),
            )
            .clicked()
        {
            generator_tool!(app, tool);
        }
    });

    false
}

fn source_ui(app: &mut PopupApp, ui: &mut Ui, id: SourceId) {
    ui.horizontal(|ui| {
        let source = app.state.tools.sliced_diff.source_mut(id);

        if let Some(result) = app.slice_operation.as_ref().map(|x| x.result())
            && let Some(result) = result.as_ref()
            && let GenericSliceData::Raster { data, .. } = result.slice_data()
        {
            if ui
                .button(STACK_SIMPLE)
                .on_hover_text("Just sliced")
                .clicked()
            {
                *source = Source::Loaded {
                    config: result.config.clone(),
                    layers: Arc::new(data),
                };
            }
        } else {
            ui.add_enabled(false, Button::new(STACK_SIMPLE));
        }

        if ui
            .button(FOLDER_OPEN)
            .on_hover_text("Load from disk")
            .clicked()
        {
            app.tasks.add(FileDialog::pick_file(
                ("Sliced", &["goo", "ctb", "nanodlp"]),
                move |app, path, _tasks| {
                    let file = File::open(path).unwrap();
                    let ext = path.extension().unwrap().to_string_lossy();
                    let format = RasterFormat::from_extension(&ext).unwrap();

                    *app.state.tools.sliced_diff.source_mut(id) = Source::File {
                        path: path.to_path_buf(),
                        file: Arc::new(util::load_sliced(&format, file).unwrap()),
                    };
                },
            ));
        }

        match source {
            Source::Empty => {
                ui.label("Unspecified");
            }
            Source::File { path, file } => {
                let name = path.file_name().unwrap().to_string_lossy();
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label(RichText::new(name).underline())
                        .on_hover_text(path.to_string_lossy());
                    ui.label(format!(" ({} layers)", file.layer_count()));
                });
            }
            Source::Loaded { layers, .. } => {
                ui.label(format!("Loaded ({} layers)", layers.len()));
            }
        };
    });
}
