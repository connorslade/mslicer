use egui::{pos2, vec2, Color32, Context, Grid, Id, Label, RichText, Ui, WidgetText, Window};

use crate::app::App;

type UiFunction = dyn FnMut(&mut App, &mut Ui) -> bool;
pub struct PopupManager {
    popups: Vec<Popup>,
}

pub struct Popup {
    title: String,
    id: Id,
    ui: Box<UiFunction>,
}

#[allow(dead_code)]
pub enum PopupIcon {
    Info,
    Warning,
    Error,
    Success,
}

impl PopupManager {
    pub fn new() -> Self {
        Self { popups: Vec::new() }
    }

    pub fn open(&mut self, popup: Popup) {
        self.popups.push(popup);
    }

    pub fn render(&mut self, app: &mut App, ctx: &Context) {
        let window_size = ctx.screen_rect().size();

        let mut i = 0;
        let mut close = false;

        while i < self.popups.len() {
            let popup = &mut self.popups[i];
            Window::new("")
                .id(popup.id)
                .title_bar(false)
                .resizable(false)
                .default_size(vec2(400.0, 0.0))
                .default_pos(pos2(
                    window_size.x * 0.5 - 200.0,
                    window_size.y * 0.5 - 100.0,
                ))
                .show(ctx, |ui| {
                    ui.set_height(50.0);
                    ui.vertical_centered(|ui| {
                        ui.heading(popup.title.clone());
                    });
                    ui.separator();
                    close = (popup.ui)(app, ui);
                });

            if close {
                self.popups.remove(i);
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
        ui: impl FnMut(&mut App, &mut Ui) -> bool + 'static,
    ) -> Self {
        Self {
            title,
            id,
            ui: Box::new(ui),
        }
    }

    pub fn new(
        title: impl AsRef<str>,
        ui: impl FnMut(&mut App, &mut Ui) -> bool + 'static,
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
                    ui.add(Label::new(body.clone()).wrap(true));
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
