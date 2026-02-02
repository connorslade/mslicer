use egui::{Color32, Context, Grid, Id, Label, RichText, Ui, WidgetText, Window, vec2};

use crate::{
    app::{App, remote_print::RemotePrint},
    app_ref_type,
    ui::state::UiState,
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
}

pub struct PopupApp<'a> {
    pub state: &'a mut UiState,
    pub remote_print: &'a mut RemotePrint,
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
        self.popups.push(popup);
    }
}

impl<'a> PopupManagerRef<'a> {
    pub fn render(&mut self, ctx: &Context) {
        let mut i = 0;
        let mut close = false;

        let this = &mut self.app.popup;
        let mut app = PopupApp {
            state: &mut self.app.state,
            remote_print: &mut self.app.remote_print,
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
                    ui.set_height(50.0);
                    ui.vertical_centered(|ui| {
                        ui.heading(popup.title.clone());
                    });
                    ui.separator();
                    close = (popup.ui)(&mut app, ui);
                });

            if close {
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

    pub fn simple(title: impl AsRef<str>, icon: PopupIcon, body: impl Into<WidgetText>) -> Self {
        let id = Id::new(rand::random::<u64>());
        let title = title.as_ref().to_owned();
        let body = body.into();

        Self::new_with_id(id, title, move |_app, ui| {
            let mut close = false;
            ui.centered_and_justified(|ui| {
                Grid::new(id.with("grid")).num_columns(2).show(ui, |ui| {
                    ui.label(RichText::new(icon.as_char()).size(30.0).color(icon.color()));
                    ui.add(Label::new(body.clone()).wrap());
                });
                ui.add_space(5.0);
                close = ui.button("Close").clicked();
            });
            close
        })
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
