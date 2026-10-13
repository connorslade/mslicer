use egui::{Context, Id, Ui, WidgetText};
use egui_dock::TabViewer;
use serde::{Deserialize, Serialize};

use crate::core::App;
use crate::interface::panels::*;

pub struct Tabs<'a> {
    pub app: &'a mut App,
    pub ctx: &'a Context,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tab {
    Logs,
    Models,
    RemotePrint,
    SliceConfig,
    Sliced,
    Supports,
    Viewport,
    Workspace,
    Notes,
}

impl Tab {
    pub const ALL: [Tab; 8] = [
        Self::Logs,
        Self::Models,
        Self::Notes,
        Self::RemotePrint,
        Self::SliceConfig,
        Self::Sliced,
        Self::Supports,
        Self::Workspace,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Logs => "Logs",
            Self::Models => "Models",
            Self::Notes => "Notes",
            Self::RemotePrint => "Remote Print",
            Self::SliceConfig => "Slice Config",
            Self::Sliced => "Sliced",
            Self::Supports => "Supports",
            Self::Viewport => "Viewport",
            Self::Workspace => "Workspace",
        }
    }
}

impl TabViewer for Tabs<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Logs => logs::ui(self.app, ui, self.ctx),
            Tab::Models => models::ui(self.app, ui, self.ctx),
            Tab::Notes => notes::ui(self.app, ui, self.ctx),
            Tab::RemotePrint => remote_print::ui(self.app, ui, self.ctx),
            Tab::SliceConfig => slice_config::ui(self.app, ui, self.ctx),
            Tab::Sliced => sliced::ui(self.app, ui, self.ctx),
            Tab::Supports => supports::ui(self.app, ui, self.ctx),
            Tab::Viewport => viewport::ui(self.app, ui, self.ctx),
            Tab::Workspace => workspace::ui(self.app, ui, self.ctx),
        }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        *tab != Tab::Viewport
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        *tab != Tab::Viewport
    }

    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(tab)
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, true]
    }
}
