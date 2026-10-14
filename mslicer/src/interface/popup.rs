use std::{mem, path::PathBuf};

use egui::{
    Align, Button, Color32, Context, Grid, Id, Label, Layout, RichText, Ui, UiBuilder, Widget,
    WidgetText, Window, vec2,
};
use egui_phosphor::regular::X;
use remote_print::manager::RemotePrintManager;

use crate::{
    app_ref_type,
    core::{
        App, config::Config, history::History, is_slicing, project::Project, slice::SliceOperation,
        state::UiState,
    },
    interface::{components::button_row, panels::Panels},
    render::camera::Camera,
    task::{Task, TaskApp, TaskManager},
};

type UiFunction = dyn FnMut(&mut PopupApp, &mut Ui) -> bool;

#[derive(Default)]
pub struct PopupManager {
    popups: Vec<Popup>,
}

app_ref_type!(PopupManager, popup);

pub struct Popup {
    title: String,
    id: Id,
    ui: Box<UiFunction>,

    close_button: bool,
    height: Option<f32>,
}

pub struct PopupApp<'a> {
    pub config_dir: &'a mut PathBuf,
    pub panels: &'a mut Panels,
    pub tasks: &'a mut TaskManager,
    pub remote_print: &'a mut RemotePrintManager,
    pub slice_operation: &'a mut Option<SliceOperation>,
    pub camera: &'a mut Camera,
    pub state: &'a mut UiState,
    pub config: &'a mut Config,
    pub project: &'a mut Project,
    pub history: &'a mut History,
}

#[allow(dead_code)]
pub enum PopupIcon {
    Info,
    Warning,
    Error,
    Success,
}

impl PopupManager {
    pub fn open(&mut self, popup: Popup) {
        if self.popups.iter().any(|x| x.id == popup.id) {
            return;
        }

        self.popups.push(popup);
    }
}

impl<'a> PopupManagerRef<'a> {
    pub fn render(&mut self, ctx: &Context) {
        let mut i = 0;
        let mut close = false;

        let this = &mut self.app.popup;
        let mut app = PopupApp {
            config_dir: &mut self.app.config_dir,
            panels: &mut self.app.panels,
            tasks: &mut self.app.tasks,
            remote_print: &mut self.app.remote_print,
            slice_operation: &mut self.app.slice_operation,
            camera: &mut self.app.camera,
            state: &mut self.app.state,
            config: &mut self.app.config,
            project: &mut self.app.project,
            history: &mut self.app.history,
        };

        while i < this.popups.len() {
            let popup = &mut this.popups[i];
            let size = vec2(400.0, 0.0);
            Window::new("")
                .id(popup.id)
                .title_bar(false)
                .resizable(false)
                .default_size(size)
                .default_pos((ctx.content_rect().size() - size).to_pos2() / 2.0)
                .show(ctx, |ui| {
                    if let Some(height) = popup.height {
                        ui.set_height(height);
                    }

                    let title = ui.vertical_centered(|ui| {
                        ui.heading(popup.title.clone());
                    });

                    if popup.close_button {
                        ui.scope_builder(
                            UiBuilder::new()
                                .max_rect(title.response.rect)
                                .layout(Layout::right_to_left(Align::Center)),
                            |ui| {
                                ui.add_space(4.0);
                                close |= Button::new(X).frame(false).ui(ui).clicked();
                            },
                        );
                    }

                    ui.separator();
                    close |= (popup.ui)(&mut app, ui);
                });

            if mem::take(&mut close) {
                this.popups.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

impl Popup {
    fn new_with_id(
        id: Id,
        title: String,
        ui: impl FnMut(&mut PopupApp, &mut Ui) -> bool + 'static,
    ) -> Self {
        Self {
            title,
            id,
            ui: Box::new(ui),
            close_button: false,
            height: None,
        }
    }

    pub fn new(
        title: impl AsRef<str>,
        ui: impl FnMut(&mut PopupApp, &mut Ui) -> bool + 'static,
    ) -> Self {
        Self::new_with_id(
            Id::new(rand::random::<u64>()),
            title.as_ref().to_owned(),
            ui,
        )
    }

    pub fn new_seeded(
        title: impl AsRef<str>,
        ui: impl FnMut(&mut PopupApp, &mut Ui) -> bool + 'static,
    ) -> Self {
        Self::new_with_id(Id::new(title.as_ref()), title.as_ref().to_owned(), ui)
    }

    pub fn simple(title: impl AsRef<str>, icon: PopupIcon, body: impl Into<WidgetText>) -> Self {
        let body = body.into();
        Self::new(title, move |_app, ui| {
            let mut close = false;
            ui.centered_and_justified(|ui| {
                Grid::new(ui.id().with("grid"))
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(RichText::new(icon.as_char()).size(30.0).color(icon.color()));
                        ui.add(Label::new(body.clone()).wrap());
                    });
                ui.add_space(5.0);
                close = ui.button("Close").clicked();
            });
            close
        })
        .height(50.0)
    }

    pub fn close_button(mut self, close_button: bool) -> Self {
        self.close_button = close_button;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
}

impl PopupIcon {
    pub fn as_char(&self) -> char {
        match self {
            Self::Info => 'ℹ',
            Self::Warning => '⚠',
            Self::Error => '❌',
            Self::Success => '✔',
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Info => Color32::from_rgb(150, 200, 210),
            Self::Warning => Color32::from_rgb(230, 220, 140),
            Self::Error => Color32::from_rgb(200, 90, 90),
            Self::Success => Color32::from_rgb(140, 230, 140),
        }
    }
}

impl<'a> PopupApp<'a> {
    // i know, i know…
    pub fn is_slicing(&self) -> bool {
        is_slicing(self.slice_operation)
    }
}

// what r we even doing at this point...
#[allow(unused)]
pub struct ConfirmUnsavedApp<'a> {
    pub config_dir: &'a mut PathBuf, // only used on windows
    pub config: &'a mut Config,
    pub project: &'a mut Project,
    pub history: &'a mut History,
    pub state: &'a mut UiState,
}

impl<'a> ConfirmUnsavedApp<'a> {
    pub fn from_app(app: &'a mut App) -> Self {
        Self {
            config_dir: &mut app.config_dir,
            config: &mut app.config,
            project: &mut app.project,
            history: &mut app.history,
            state: &mut app.state,
        }
    }

    pub fn from_popup_app(app: &'a mut PopupApp) -> Self {
        Self {
            config_dir: &mut app.config_dir,
            config: &mut app.config,
            project: &mut app.project,
            history: &mut app.history,
            state: &mut app.state,
        }
    }

    pub fn from_task_app(app: &'a mut TaskApp) -> Self {
        Self {
            config_dir: &mut app.config_dir,
            config: &mut app.config,
            project: &mut app.project,
            history: &mut app.history,
            state: &mut app.state,
        }
    }
}

// todo: add dirty flag to history
pub fn confirm_unsaved(
    app: &mut App,
    mut callback: impl FnMut(&mut ConfirmUnsavedApp) -> Vec<Box<dyn Task>> + 'static,
) {
    if app.project.models.is_empty() {
        let tasks = callback(&mut ConfirmUnsavedApp::from_app(app));
        app.tasks.extend_boxed(tasks);
        return;
    }

    let mut callback = Some(callback);
    app.popup
        .open(Popup::new_seeded("Unsaved Changes", move |app, ui| {
            ui.add_space(8.0);
            ui.label(if app.project.path.is_some() {
                "Do you want to save the changes made to this project?"
            } else {
                "Do you want to save this project?"
            });
            ui.add_space(8.0);

            let (mut cancel, mut dont_save, mut save) = (false, false, false);
            button_row(
                ui,
                [
                    ("Cancel", &mut || cancel = true),
                    ("Don't Save", &mut || dont_save = true),
                    ("Save", &mut || save = true),
                ],
            );

            if save && let Some(mut callback) = callback.take() {
                app.tasks.add_boxed(
                    app.project
                        .save(move |app| callback(&mut ConfirmUnsavedApp::from_task_app(app))),
                );
            } else if dont_save && let Some(mut callback) = callback.take() {
                let tasks = callback(&mut ConfirmUnsavedApp::from_popup_app(app));
                app.tasks.extend_boxed(tasks);
            }

            cancel || dont_save || save
        }));
}
